# Windows Camera Driver

iMirror V2 includes an experimental DirectShow camera path for Windows.

What is included:

- `UnityCaptureFilter64.dll` for modern Windows x64
- `Install-iMirrorCamera.ps1`
- `Uninstall-iMirrorCamera.ps1`
- A Tauri/Rust shared-memory frame publisher in the desktop app

The driver registers as `iMirror Camera`. When Chrome, OBS, Zoom, or another app opens that camera, iMirror Desktop writes the connected phone stream into UnityCapture shared memory.

## Install

Open PowerShell as Administrator from the repo root:

```powershell
pnpm install:windows-camera
```

Then restart Chrome or any app that had its camera picker open.

The installer automatically removes Windows downloaded-file blocks from the bundled DLLs before registration. It
registers the UnityCapture filters with plain `regsvr32`, then renames the registered DirectShow entries to
`iMirror Camera`. If `regsvr32` still fails, make sure this repo is fully synced locally in OneDrive and run the
command again from an Administrator PowerShell window.

## Test

1. Start iMirror Desktop.
2. Connect your phone.
3. Open `TEST-CAMERAS.html` in Chrome.
4. Click `Scan cameras`.
5. Select `iMirror Camera`.
6. Click `Start selected camera`.

The iMirror app status should move from `Waiting` to `Streaming`.

## Uninstall

Open PowerShell as Administrator from the repo root:

```powershell
pnpm uninstall:windows-camera
```

Restart Chrome after uninstalling.

## Notes

- This is a DirectShow virtual camera based on UnityCapture, not a Windows Media Foundation virtual camera.
- Some apps cache camera devices. Restart the app after install or uninstall.
- OBS is no longer required for the direct Windows path, but OBS Output Mode remains available as a fallback.
