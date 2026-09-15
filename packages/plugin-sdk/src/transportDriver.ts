import type { iMirrorTransport } from "@imirror/shared";
import type { iMirrorPlugin } from "./pluginTypes";

export interface TransportDriverPlugin extends iMirrorPlugin {
  type: "transport-driver";
  discover(): Promise<iMirrorTransport[]>;
}
