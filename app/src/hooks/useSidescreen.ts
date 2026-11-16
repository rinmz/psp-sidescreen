import { useCallback, useEffect, useRef, useState } from "react";

import {
  DEFAULT_SETTINGS,
  createBackend,
  type Backend,
  type Monitor,
  type SessionState,
  type Settings,
} from "../lib/backend";

export interface Sidescreen {
  ready: boolean;
  settings: Settings;
  monitors: Monitor[];
  state: SessionState;
  version: string;
  /** Changes one or more settings and remembers them. */
  update: (change: Partial<Settings>) => void;
  start: () => void;
  stop: () => void;
}

/** The state of the window: the settings, the monitors, and where the session has got to. */
export function useSidescreen(): Sidescreen {
  const backend = useRef<Backend | null>(null);
  const [ready, setReady] = useState(false);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const [state, setState] = useState<SessionState>({ kind: "idle" });
  const [version, setVersion] = useState("");

  useEffect(() => {
    let stopListening: (() => void) | undefined;
    let cancelled = false;

    void (async () => {
      const created = await createBackend();
      backend.current = created;
      const [loaded, found, number] = await Promise.all([
        created.getSettings(),
        created.listMonitors(),
        created.version(),
      ]);
      const unlisten = await created.onState(setState);
      if (cancelled) {
        unlisten();
        return;
      }
      stopListening = unlisten;
      setSettings(loaded);
      setMonitors(found);
      setVersion(number);
      setReady(true);
    })();

    return () => {
      cancelled = true;
      stopListening?.();
    };
  }, []);

  const update = useCallback((change: Partial<Settings>) => {
    setSettings((current) => {
      const next = { ...current, ...change };
      void backend.current?.saveSettings(next);
      return next;
    });
  }, []);

  const start = useCallback(() => void backend.current?.start(), []);
  const stop = useCallback(() => void backend.current?.stop(), []);

  return { ready, settings, monitors, state, version, update, start, stop };
}
