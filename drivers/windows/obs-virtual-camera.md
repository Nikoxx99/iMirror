# OBS Virtual Camera on Windows

This is the fallback path. Prefer `iMirror Camera` after installing the Windows DirectShow driver with:

```powershell
pnpm install:windows-camera
```

Fallback steps:

1. Install OBS Studio.
2. In iMirror, click **Open OBS Output**.
3. Add **iMirror OBS Output** as a Window Capture source.
4. Start OBS Virtual Camera.
5. Select OBS Virtual Camera in your meeting app.

If OBS shows **iMirror Desktop**, go back to iMirror and open OBS Output first.
