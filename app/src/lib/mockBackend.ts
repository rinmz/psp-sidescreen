import {
  DEFAULT_SETTINGS,
  type Backend,
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
