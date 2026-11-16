import {
  DEFAULT_SETTINGS,
  type Backend,
  type ControlCatalog,
  type Monitor,
  type SessionState,
  type Settings,
} from "./backend";

/**
 * Stands in for the program when the page is opened in a plain browser, so the window can be
 * designed and checked without a PSP. It plays the states a real session goes through.
 */
const STORAGE_KEY = "sidescreen-settings";
const monitors: Monitor[] = [
  { index: 0, name: "\\\\.\\DISPLAY1", width: 1920, height: 1080 },
];

const catalog: ControlCatalog = {
  buttons: [
    { id: "cross", label: "Cross" },
    { id: "circle", label: "Circle" },
    { id: "square", label: "Square" },
    { id: "triangle", label: "Triangle" },
    { id: "up", label: "D-pad up" },
    { id: "down", label: "D-pad down" },
    { id: "left", label: "D-pad left" },
    { id: "right", label: "D-pad right" },
    { id: "l", label: "L trigger" },
    { id: "r", label: "R trigger" },
    { id: "start", label: "Start" },
    { id: "select", label: "Select" },
  ],
  actions: [
    { id: "nothing", label: "Nothing" },
    { id: "enter", label: "Enter" },
    { id: "escape", label: "Escape" },
    { id: "switch_window", label: "Switch window (Alt+Tab)" },
    { id: "play_pause", label: "Play or pause music" },
    { id: "volume_up", label: "Volume up" },
    { id: "volume_down", label: "Volume down" },
    { id: "left_click", label: "Left click" },
    { id: "right_click", label: "Right click" },
    { id: "scroll_up", label: "Scroll up" },
    { id: "scroll_down", label: "Scroll down" },
  ],
};

const listeners = new Set<(state: SessionState) => void>();
let timers: number[] = [];

function emit(state: SessionState) {
  listeners.forEach((listener) => listener(state));
}

function clearTimers() {
  timers.forEach((timer) => window.clearTimeout(timer));
  timers = [];
}

function readSettings(): Settings {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored)
      return { ...DEFAULT_SETTINGS, ...(JSON.parse(stored) as Partial<Settings>) };
  } catch {
    // Storage can be blocked; the defaults are fine.
  }
  return DEFAULT_SETTINGS;
}

export const mockBackend: Backend = {
  getSettings: () => Promise.resolve(readSettings()),
  saveSettings: (settings) => {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    } catch {
      // Not remembered; nothing else depends on it.
    }
    return Promise.resolve();
  },
  listMonitors: () => Promise.resolve(monitors),
  listControls: () => Promise.resolve(catalog),
  start: () => {
    clearTimers();
    emit({ kind: "starting" });
    timers.push(window.setTimeout(() => emit({ kind: "waiting" }), 900));
    timers.push(
      window.setTimeout(
        () => emit({ kind: "streaming", fps: 30, width: 1366, height: 768 }),
        2600,
      ),
    );
    return Promise.resolve();
  },
  stop: () => {
    clearTimers();
    emit({ kind: "idle" });
    return Promise.resolve();
  },
  onState: (listener) => {
    listeners.add(listener);
    return Promise.resolve(() => {
      listeners.delete(listener);
    });
  },
  version: () => Promise.resolve("0.1.0"),
};
