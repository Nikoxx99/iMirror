import type { iMirrorSource } from "@imirror/shared";

export interface SourceStoreSnapshot {
  sources: iMirrorSource[];
  activeSourceId: string | null;
}

export const emptySourceSnapshot: SourceStoreSnapshot = {
  sources: [],
  activeSourceId: null
};
