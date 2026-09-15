export type iMirrorPluginType = "video-filter" | "source-driver" | "transport-driver" | "ui-panel";

export interface PluginContext {
  log(message: string, metadata?: Record<string, unknown>): void;
  version: string;
}

export interface iMirrorPlugin {
  name: string;
  version: string;
  type: iMirrorPluginType;
  setup?: (context: PluginContext) => void | Promise<void>;
  dispose?: () => void | Promise<void>;
}
