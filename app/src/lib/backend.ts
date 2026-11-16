/** What the window asks of the program behind it, and what it hears back. */

export type Quality = "lossless" | "high" | "medium" | "low";
export type Mode = "extended" | "mirror";

export interface Settings {
  /** `extended`: the PSP gets a screen of its own. `mirror`: it shows an existing monitor. */
  mode: Mode;
  /** Size of the PSP's own screen, such as `1366x768`; used when `mode` is `extended`. */
  size: string;
  /** Index of the monitor to show; used when `mode` is `mirror`. */
  monitor: number;
  quality: Quality;
}

export interface Monitor {
  index: number;
  name: string;
  width: number;
  height: number;
}

export type SessionState =
  | { kind: "idle" }
  | { kind: "starting" }
  /** The host is running but the PSP is not answering yet. */
  | { kind: "waiting" }
  | { kind: "streaming"; fps: number; width: number; height: number }
  /** The monitor went away (Win+P, a resolution change) and is being looked for. */
  | { kind: "reconnecting" }
  | { kind: "error"; message: string };

export interface Backend {
  getSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<void>;
  listMonitors(): Promise<Monitor[]>;
  start(): Promise<void>;
  stop(): Promise<void>;
  /** Calls `listener` with every change of state; returns a function that stops listening. */
  onState(listener: (state: SessionState) => void): Promise<() => void>;
  version(): Promise<string>;
}

export const DEFAULT_SETTINGS: Settings = {
  mode: "extended",
  size: "1366x768",
  monitor: 0,
  quality: "medium",
};

/** Sizes the virtual display driver offers. */
export const SIZES = [
  "800x600",
  "1366x768",
  "1920x1080",
  "2560x1440",
  "3840x2160",
] as const;

/** The real backend inside the Tauri window, the stand-in in a plain browser. */
export async function createBackend(): Promise<Backend> {
  if ("__TAURI_INTERNALS__" in window) {
    return (await import("./tauriBackend")).tauriBackend;
  }
  return (await import("./mockBackend")).mockBackend;
}
