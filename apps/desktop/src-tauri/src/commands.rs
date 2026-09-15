use crate::{
    errors::IMirrorResult,
    pairing::qr_payload::PairingPayload,
    security::{allowlist::TrustedDeviceRecord, audit_log::SecurityAuditEvent},
    sources::source_manager::SourceManager,
    state::{AppState, RuntimeStatus},
    virtual_cam::{
        manager::{ObsVirtualCameraStatus, VirtualCameraManager, VirtualCameraStatus},
        unity_capture::{UnityCaptureFramePayload, UnityCapturePublishResult},
    },
};
use std::{fs, path::PathBuf, process::Command};
use tauri::{
    ipc::{InvokeBody, Request},
    AppHandle, Manager,
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustDeviceRequest {
    pub device_id: String,
    pub label: String,
    pub platform: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverActionResult {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub fn get_pairing_session(state: tauri::State<'_, AppState>) -> IMirrorResult<PairingPayload> {
    Ok(state.pairing_payload())
}

#[tauri::command]
pub fn regenerate_pairing_session(
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<PairingPayload> {
    Ok(state.regenerate_pairing_payload())
}

#[tauri::command]
pub fn get_runtime_status(state: tauri::State<'_, AppState>) -> IMirrorResult<RuntimeStatus> {
    Ok(state.runtime_status())
}

#[tauri::command]
pub fn disconnect_session(state: tauri::State<'_, AppState>) -> IMirrorResult<()> {
    state.session_manager.clear_active_device();
    Ok(())
}

#[tauri::command]
pub fn list_trusted_devices(
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<Vec<TrustedDeviceRecord>> {
    Ok(state.trusted_devices.list())
}

#[tauri::command]
pub fn is_trusted_device(
    device_id: String,
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<bool> {
    Ok(state.trusted_devices.is_trusted(&device_id))
}

#[tauri::command]
pub fn trust_device(
    request: TrustDeviceRequest,
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<TrustedDeviceRecord> {
    let record = state.trusted_devices.trust(
        request.device_id.clone(),
        request.label.clone(),
        request.platform,
        request.user_agent,
    )?;
    let _ = state.audit_log.record(
        "trusted-device-added",
        Some(request.device_id),
        Some(request.label),
        "Trusted device added.",
    );
    Ok(record)
}

#[tauri::command]
pub fn mark_trusted_device_seen(
    device_id: String,
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<()> {
    state.trusted_devices.mark_seen(&device_id)
}

#[tauri::command]
pub fn revoke_trusted_device(
    device_id: String,
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<bool> {
    let removed = state.trusted_devices.revoke(&device_id)?;
    if removed {
        let _ = state.audit_log.record(
            "trusted-device-revoked",
            Some(device_id),
            None,
            "Trusted device revoked.",
        );
    }
    Ok(removed)
}

#[tauri::command]
pub fn record_security_audit_event(
    kind: String,
    device_id: Option<String>,
    label: Option<String>,
    message: String,
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<SecurityAuditEvent> {
    state.audit_log.record(kind, device_id, label, message)
}

#[tauri::command]
pub fn list_security_audit_events(
    state: tauri::State<'_, AppState>,
) -> IMirrorResult<Vec<SecurityAuditEvent>> {
    Ok(state.audit_log.list_recent(50))
}

#[tauri::command]
pub fn list_source_statuses() -> IMirrorResult<Vec<crate::sources::source_trait::SourceDescriptor>>
{
    Ok(SourceManager::default().list())
}

#[tauri::command]
pub fn get_virtual_camera_status() -> IMirrorResult<VirtualCameraStatus> {
    Ok(VirtualCameraManager::default().status())
}

#[tauri::command]
pub fn get_obs_virtual_camera_status() -> IMirrorResult<ObsVirtualCameraStatus> {
    Ok(VirtualCameraManager::default().obs_virtual_camera_status())
}

#[tauri::command]
pub fn publish_unity_capture_frame(
    frame: UnityCaptureFramePayload,
) -> IMirrorResult<UnityCapturePublishResult> {
    crate::virtual_cam::unity_capture::publish_unity_capture_frame(frame)
}

#[tauri::command]
pub fn publish_unity_capture_frame_binary(
    request: Request<'_>,
) -> IMirrorResult<UnityCapturePublishResult> {
    match request.body() {
        InvokeBody::Raw(payload) => {
            crate::virtual_cam::unity_capture::publish_unity_capture_frame_binary(payload)
        }
        InvokeBody::Json(_) => Err(crate::errors::IMirrorError::new(
            "virtual_cam_bad_frame",
            "Binary frame command expected a raw IPC payload.",
        )),
    }
}

#[tauri::command]
pub fn reset_unity_capture_bridge() -> IMirrorResult<()> {
    crate::virtual_cam::unity_capture::reset_unity_capture_bridge()
}

#[tauri::command]
pub fn install_windows_camera(app: AppHandle) -> IMirrorResult<DriverActionResult> {
    run_camera_driver_script(&app, "Install-iMirrorCamera.ps1", "installed")
}

#[tauri::command]
pub fn uninstall_windows_camera(app: AppHandle) -> IMirrorResult<DriverActionResult> {
    run_camera_driver_script(&app, "Uninstall-iMirrorCamera.ps1", "removed")
}

#[cfg(target_os = "windows")]
fn run_camera_driver_script(
    app: &AppHandle,
    script_name: &str,
    completed_action: &str,
) -> IMirrorResult<DriverActionResult> {
    let driver_directory = bundled_driver_directory(app)?;
    let script = driver_directory.join(script_name);
    if !script.is_file() {
        return Err(crate::errors::IMirrorError::new(
            "camera-driver-missing",
            "The bundled iMirror Camera installer is missing.",
        )
        .with_detail(script.display().to_string()));
    }

    let script_arg = powershell_literal(&script);
    let directory_arg = powershell_literal(&driver_directory);
    let log_path = std::env::temp_dir().join(format!(
        "imirror-camera-driver-{}-{}.log",
        std::process::id(),
        completed_action
    ));
    let log_arg = powershell_literal(&log_path);
    let elevated = format!(
        "$ErrorActionPreference='Stop'; Unblock-File -LiteralPath {script_arg} -ErrorAction SilentlyContinue; $p=Start-Process -FilePath 'powershell.exe' -Verb RunAs -Wait -PassThru -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',{script_arg},'-DriverPath',{directory_arg},'-LogPath',{log_arg}); exit $p.ExitCode"
    );
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &elevated,
        ])
        .output()
        .map_err(|error| {
            crate::errors::IMirrorError::new(
                "camera-driver-launch-failed",
                "Could not open the iMirror Camera installer.",
            )
            .with_detail(error.to_string())
        })?;

    if !output.status.success() {
        let mut detail = String::from_utf8_lossy(&output.stdout).into_owned();
        detail.push_str(&String::from_utf8_lossy(&output.stderr));
        if let Ok(log) = fs::read_to_string(&log_path) {
            detail.push_str(&log);
        }
        let mut error = crate::errors::IMirrorError::new(
            "camera-driver-action-failed",
            "The camera driver action was cancelled or failed.",
        )
        .with_fix("Accept the Windows administrator prompt and review the diagnostic details.");
        if let Some(detail) = compact_driver_diagnostic(&detail) {
            error = error.with_detail(detail);
        }
        return Err(error);
    }

    Ok(DriverActionResult {
        success: true,
        message: format!(
            "iMirror Camera was {completed_action}. Restart apps that already had their camera list open."
        ),
    })
}

fn compact_driver_diagnostic(detail: &str) -> Option<String> {
    let trimmed = detail.trim();
    if trimmed.is_empty() {
        return None;
    }

    let compact: String = trimmed.chars().take(6000).collect();
    if compact.chars().count() < trimmed.chars().count() {
        Some(format!("{compact}\n[diagnostic output truncated]"))
    } else {
        Some(compact)
    }
}

#[cfg(not(target_os = "windows"))]
fn run_camera_driver_script(
    _app: &AppHandle,
    _script_name: &str,
    _completed_action: &str,
) -> IMirrorResult<DriverActionResult> {
    Err(crate::errors::IMirrorError::new(
        "camera-driver-unsupported",
        "iMirror Camera is available only on Windows 11 x64.",
    ))
}

fn bundled_driver_directory(app: &AppHandle) -> IMirrorResult<PathBuf> {
    let resource_directory = app.path().resource_dir().map_err(|error| {
        crate::errors::IMirrorError::new(
            "resource-directory-unavailable",
            "Could not locate iMirror's installed resources.",
        )
        .with_detail(error.to_string())
    })?;
    Ok(strip_extended_windows_prefix(resource_directory).join("drivers"))
}

fn strip_extended_windows_prefix(path: PathBuf) -> PathBuf {
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = value.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path
}

fn powershell_literal(path: &std::path::Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::strip_extended_windows_prefix;
    use std::path::PathBuf;

    #[test]
    fn strips_extended_drive_prefix() {
        let path = PathBuf::from(r"\\?\C:\Users\Usuario\AppData\Local\iMirror");
        assert_eq!(
            strip_extended_windows_prefix(path),
            PathBuf::from(r"C:\Users\Usuario\AppData\Local\iMirror")
        );
    }

    #[test]
    fn converts_extended_unc_prefix() {
        let path = PathBuf::from(r"\\?\UNC\server\share\iMirror");
        assert_eq!(
            strip_extended_windows_prefix(path),
            PathBuf::from(r"\\server\share\iMirror")
        );
    }
}
