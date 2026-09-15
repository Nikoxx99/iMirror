export type IMirrorErrorCode =
  | "camera-permission-denied"
  | "camera-unavailable"
  | "browser-unsupported"
  | "insecure-context"
  | "network-unreachable"
  | "desktop-unavailable"
  | "invalid-pairing-token"
  | "expired-session"
  | "webrtc-failed"
  | "signaling-failed"
  | "virtual-camera-unavailable"
  | "not-implemented";

export interface IMirrorErrorShape {
  code: IMirrorErrorCode;
  message: string;
  technicalDetail?: string;
  suggestedFix?: string;
}
