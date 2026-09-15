export const OBS_OUTPUT_TITLE = "iMirror OBS Output";

export const OBS_SETUP_STEPS = `iMirror OBS Fallback Setup

1. Connect your phone in iMirror.
2. First try iMirror Camera directly in Chrome, OBS, or your target app.
3. If the app will not use iMirror Camera, click "Open OBS Output".
4. Open OBS Studio.
5. Add Source -> Window Capture.
6. Select "iMirror OBS Output".
   If OBS says "iMirror Desktop", go back to iMirror and click "Open OBS Output" first.
7. If the preview is black, open source properties and change Capture Method:
   - Try Windows Graphics Capture
   - Then Windows 10 1903 and up
   - Then BitBlt
8. Right-click the source -> Transform -> Fit to Screen.
9. Click "Start Virtual Camera" in OBS.
10. Refresh or restart Chrome, then open your browser/app camera settings.
11. Select "OBS Virtual Camera".

Important: iMirror Camera is the primary Windows output. OBS is only the fallback bridge.`;

export const OBS_CAPTURE_WINDOW_LABEL = "[imirror-desktop.exe]: iMirror OBS Output";
