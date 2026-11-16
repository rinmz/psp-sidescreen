import { useCallback, useEffect, useRef, useState } from "react";

import {
  DEFAULT_SETTINGS,
  createBackend,
  type Backend,
  type ControlCatalog,
  type DriverOutcome,
  type Monitor,
  type PspInstall,
  type SessionState,
  type Settings,
  type SetupStatus,
} from "../lib/backend";

export interface Sidescreen {
  ready: boolean;
  settings: Settings;
  monitors: Monitor[];
  catalog: ControlCatalog;
  state: SessionState;
  version: string;
  /** Changes one or more settings and remembers them. */
  update: (change: Partial<Settings>) => void;
  start: () => void;
  stop: () => void;
  /** Looks at what is installed and whether a PSP is connected. */
  getSetup: () => Promise<SetupStatus>;
  installPspProgram: () => Promise<PspInstall>;
  installPspDriver: () => Promise<DriverOutcome>;
}

/** The backend, which exists once the window has loaded; the pages are shown only then. */
function requireBackend(backend: { current: Backend | null }): Backend {
  if (!backend.current)
    throw new Error("the window used the program before it was ready");
  return backend.current;
}

/** The state of the window: the settings, the monitors, and where the session has got to. */
export function useSidescreen(): Sidescreen {
  const backend = useRef<Backend | null>(null);
  const [ready, setReady] = useState(false);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [monitors, setMonitors] = useState<Monitor[]>([]);
  const [catalog, setCatalog] = useState<ControlCatalog>({ buttons: [], actions: [] });
  const [state, setState] = useState<SessionState>({ kind: "idle" });
  const [version, setVersion] = useState("");

  useEffect(() => {
    let stopListening: (() => void) | undefined;
    let cancelled = false;

    void (async () => {
      const created = await createBackend();
      backend.current = created;
      const [loaded, found, controls, number] = await Promise.all([
        created.getSettings(),
        created.listMonitors(),
        created.listControls(),
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
      setCatalog(controls);
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
  const getSetup = useCallback(() => requireBackend(backend).getSetup(), []);
  const installPspDriver = useCallback(
    () => requireBackend(backend).installPspDriver(),
    [],
  );
  const installPspProgram = useCallback(
    () => requireBackend(backend).installPspProgram(),
    [],
  );

  return {
    ready,
    settings,
    monitors,
    catalog,
    state,
    version,
    update,
    start,
    stop,
    getSetup,
    installPspProgram,
    installPspDriver,
  };
}
