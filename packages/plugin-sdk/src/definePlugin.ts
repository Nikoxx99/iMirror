import type { iMirrorPlugin } from "./pluginTypes";

export function definePlugin<TPlugin extends iMirrorPlugin>(plugin: TPlugin): TPlugin {
  return plugin;
}
