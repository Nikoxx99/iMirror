# Third-Party Notices

## UnityCapture

The Windows DirectShow filter DLLs in this directory are UnityCapture filter binaries:

- `UnityCaptureFilter64.dll`

Source: https://github.com/schellingb/UnityCapture

UnityCapture is split into separately licensed parts. The `UnityCaptureFilter` DirectShow filter is MIT licensed in the upstream repository. iMirror uses the 64-bit filter as the Windows DirectShow device layer and writes frames through UnityCapture's documented shared-memory objects. The upstream MIT license and copyright notice are preserved in this repository.
