import type { PairingPayload } from "@imirror/shared";

export interface PairingState {
  payload: PairingPayload | null;
  error: string | null;
}
