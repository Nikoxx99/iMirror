import type { iMirrorPlugin } from "./pluginTypes";

export interface UiPanelPlugin extends iMirrorPlugin {
  type: "ui-panel";
  mount(element: HTMLElement): void | Promise<void>;
}
