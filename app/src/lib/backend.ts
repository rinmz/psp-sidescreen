/** What the window asks of the program behind it, and what it hears back. */

export type Quality = "lossless" | "high" | "medium" | "low";
export type Mode = "extended" | "mirror";

/** What the PSP's buttons and stick do on the PC. */
export interface ControlSettings {
  /** Off until the person turns it on. */
  enabled: boolean;
  /** Whether the analog stick moves the mouse pointer. */
  analogMouse: boolean;
  /** Action id by button id; a button that is missing does nothing. */
  buttons: Record<string, string>;
}

/** A button or an action the window offers. */
export interface Choice {
  id: string;
  label: string;
}

/** What can be chosen in the controls; the program supplies it. */
export interface ControlCatalog {
  buttons: Choice[];
  actions: Choice[];
}

/** What the Setup page shows about the machine and the PSP. */
export interface SetupStatus {
  /** Whether the virtual display driver is installed. */
  virtualDisplay: boolean;
  pspDriver: "ready" | "needs-driver" | "not-connected";
  /** The drive of a connected PSP Memory Stick, such as `E:\`; null when there is none. */
  memoryStick: string | null;
  /** Whether the PSP program is on that Memory Stick already. */
  programOnStick: boolean;
}

/** What putting the program on the PSP did. */
export interface PspInstall {
  drive: string;
  files: number;
  keptSettings: boolean;
}

export interface Settings {
  /** `extended`: the PSP gets a screen of its own. `mirror`: it shows an existing monitor. */
  mode: Mode;
  /** Size of the PSP's own screen, such as `1366x768`; used when `mode` is `extended`. */
  size: string;
  /** Index of the monitor to show; used when `mode` is `mirror`. */
  monitor: number;
  quality: Quality;
  controls: ControlSettings;
  /** Whether the person has been through the Setup page; until then it is shown first. */
  setupDone: boolean;
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
  listControls(): Promise<ControlCatalog>;
  getSetup(): Promise<SetupStatus>;
  /** Puts the PSP program on the Memory Stick; rejects with a message for the person. */
  installPspProgram(): Promise<PspInstall>;
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
  controls: { enabled: false, analogMouse: false, buttons: {} },
  setupDone: false,
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
