import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";

import type { Backend, SessionState } from "./backend";

/** The commands and the `session-state` event are provided by the `sidescreen-app` crate. */
export const tauriBackend: Backend = {
  getSettings: () => invoke("get_settings"),
  saveSettings: (settings) => invoke("save_settings", { settings }),
  listMonitors: () => invoke("list_monitors"),
  listControls: () => invoke("list_controls"),
  start: () => invoke("start_session"),
  stop: () => invoke("stop_session"),
  onState: (listener) =>
    listen<SessionState>("session-state", (event) => listener(event.payload)),
  version: () => getVersion(),
};
