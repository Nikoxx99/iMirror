import type { iMirrorPlugin } from "./pluginTypes";

export interface VideoFilterPlugin extends iMirrorPlugin {
  type: "video-filter";
  processFrame(frame: VideoFrame): VideoFrame | Promise<VideoFrame>;
}
