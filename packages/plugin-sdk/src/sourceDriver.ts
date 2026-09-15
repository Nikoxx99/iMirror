import type { iMirrorSource } from "@imirror/shared";
import type { iMirrorPlugin } from "./pluginTypes";

export interface SourceDriverPlugin extends iMirrorPlugin {
  type: "source-driver";
  discover(): Promise<iMirrorSource[]>;
}
