import type { AppConfig, Region } from "$lib/core";

export type ClickAction = "left" | "right" | "middle" | "moveOnly" | "drag";

export interface OverlayActivatePayload {
  region: Region;
  config: AppConfig;
  clickAction: ClickAction;
}

export interface NativeKeyPayload {
  key: string;
}

export interface OverlayActionPayload {
  clickAction: ClickAction;
}

export interface NativeClickPayload {
  x: number;
  y: number;
  button: ClickAction;
}
