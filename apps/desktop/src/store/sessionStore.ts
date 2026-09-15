import type { PairingPayload } from "@imirror/shared";

export interface SessionStoreSnapshot {
  activeSession: PairingPayload | null;
  connectedDeviceName: string | null;
}

export const emptySessionSnapshot: SessionStoreSnapshot = {
  activeSession: null,
  connectedDeviceName: null
};
