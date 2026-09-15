# Power-User OBS Fallback

Use OBS Virtual Camera only when the native `iMirror Camera` path is unavailable, rejected by a target app, or you want OBS scenes/filters.

1. Connect your phone in iMirror.
2. Click Open OBS Output.
3. Open OBS Studio.
4. Add a Window Capture source and select iMirror OBS Output.
   If OBS says iMirror Desktop, go back and click Open OBS Output first.
5. Right-click the source and choose Transform -> Fit to Screen.
6. Click Start Virtual Camera.
7. Refresh or restart Chrome.
8. Select OBS Virtual Camera in Chrome, Zoom, Discord, Meet, or another browser app.

If the output is reversed in the browser preview, toggle Mirror in iMirror OBS Output before OBS captures it.

If OBS does not list iMirror, the desktop window is not visible to Windows. Restart iMirror Desktop, keep it open
and unminimized, then reopen the Window Capture source properties. The entry should be
`[imirror-desktop.exe]: iMirror OBS Output`.

If OBS lists iMirror but the preview is black, try Windows Graphics Capture, Windows 10 (1903 and up), then BitBlt.
Display Capture with a crop is the fallback when Window Capture cannot see WebView2 content on a specific machine.
