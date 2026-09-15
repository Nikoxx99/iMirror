use crate::{
    errors::IMirrorResult,
    network::{local_ip::detect_local_ip, ports::find_available_port},
    pairing::{qr_payload::PairingPayload, session::SessionManager},
    security::{
        allowlist::TrustedDeviceStore,
        audit_log::SecurityAuditLog,
        tls::{load_or_create, LocalTlsIdentity},
    },
    signaling::{router::SignalingHub, server::SignalingServer},
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub session_manager: Arc<SessionManager>,
    pub signaling_hub: SignalingHub,
    pub local_host: String,
    pub signaling_port: u16,
    pub desktop_signaling_port: u16,
    pub bootstrap_port: u16,
    pub desktop_name: String,
    pub trusted_devices: Arc<TrustedDeviceStore>,
    pub audit_log: Arc<SecurityAuditLog>,
    pub tls_identity: LocalTlsIdentity,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub local_host: String,
    pub signaling_port: u16,
    pub bootstrap_port: u16,
    pub desktop_name: String,
}

impl AppState {
    pub fn new() -> IMirrorResult<Self> {
        let local_host = detect_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
        let bootstrap_port = find_available_port(48172);
        let signaling_port = find_available_port(bootstrap_port.saturating_add(1));
        let desktop_signaling_port = find_available_port(signaling_port.saturating_add(1));
        let desktop_name = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "iMirror Desktop".to_string());
        let session_manager = Arc::new(SessionManager::new(
            desktop_name.clone(),
            local_host.clone(),
            signaling_port,
            desktop_signaling_port,
            bootstrap_port,
        ));
        let tls_identity = load_or_create(&local_host)?;

        Ok(Self {
            session_manager,
            signaling_hub: SignalingHub::new(),
            local_host,
            signaling_port,
            desktop_signaling_port,
            bootstrap_port,
            desktop_name,
            trusted_devices: Arc::new(TrustedDeviceStore::new_default()),
            audit_log: Arc::new(SecurityAuditLog::new_default()),
            tls_identity,
        })
    }

    pub fn start_signaling_server(&self) {
        let secure_server = SignalingServer::new(
            self.signaling_port,
            self.session_manager.clone(),
            self.signaling_hub.clone(),
        );
        let desktop_server = SignalingServer::new(
            self.desktop_signaling_port,
            self.session_manager.clone(),
            self.signaling_hub.clone(),
        );
        let tls_identity = self.tls_identity.clone();
        let desktop_port = self.desktop_signaling_port;
        let bootstrap_port = self.bootstrap_port;
        let secure_port = self.signaling_port;
        let host = self.local_host.clone();
        let certificate_der = self.tls_identity.cert_der.clone();

        tauri::async_runtime::spawn(async move {
            if let Err(error) = secure_server.run_secure(tls_identity).await {
                eprintln!("iMirror secure signaling server failed: {error}");
            }
        });
        tauri::async_runtime::spawn(async move {
            if let Err(error) = desktop_server.run_local(desktop_port).await {
                eprintln!("iMirror desktop signaling server failed: {error}");
            }
        });
        tauri::async_runtime::spawn(async move {
            if let Err(error) =
                SignalingServer::run_bootstrap(host, bootstrap_port, secure_port, certificate_der)
                    .await
            {
                eprintln!("iMirror iPhone setup server failed: {error}");
            }
        });
    }

    pub fn runtime_status(&self) -> RuntimeStatus {
        RuntimeStatus {
            local_host: self.local_host.clone(),
            signaling_port: self.signaling_port,
            bootstrap_port: self.bootstrap_port,
            desktop_name: self.desktop_name.clone(),
        }
    }

    pub fn pairing_payload(&self) -> PairingPayload {
        self.session_manager.current_payload()
    }

    pub fn regenerate_pairing_payload(&self) -> PairingPayload {
        self.session_manager.regenerate()
    }
}
