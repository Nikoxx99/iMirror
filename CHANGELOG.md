# iMirror 0.1.2

- Fix Windows camera-driver registration when the installed resource path uses PowerShell's provider-qualified or `\\?\` extended path format.
- Normalize driver paths to native Windows paths before invoking `regsvr32`.

# iMirror 0.1.1

- Fix Windows camera-driver installation diagnostics and PowerShell elevation handling.
- Automatically unblock the bundled installer script and surface the real PowerShell error in the desktop UI.
- Keep the Windows camera package x64-only for modern Windows 11 systems.

# Changelog

## 0.1.0

- Initial V1 monorepo.
- Desktop Tauri shell and local pairing/session architecture.
- Phone PWA camera preview and WebRTC sender architecture.
- Shared protocol types and validators.
- Virtual camera, universal source, AI, audio, and plugin scaffolds.
- Open-source docs, GitHub templates, and CI scaffolding.
