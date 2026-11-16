import {
  AlertTriangle,
  Cable,
  CheckCircle2,
  Loader2,
  type LucideIcon,
} from "lucide-react";
import { useCallback, useEffect, useState, type ReactNode } from "react";

import { Button } from "../components/ui/Button";
import { SettingsGroup } from "../components/ui/SettingsGroup";
import type { Sidescreen } from "../hooks/useSidescreen";
import type { PspInstall, SetupStatus } from "../lib/backend";

/** How often the page looks again, so a PSP that is connected is noticed. */
const LOOK_EVERY_MS = 2000;

type Tone = "good" | "wait" | "bad";

const TONES: Record<Tone, { icon: LucideIcon; box: string; spin?: boolean }> = {
  good: { icon: CheckCircle2, box: "bg-moss-wash text-moss" },
  wait: { icon: Loader2, box: "bg-wash text-ink-2", spin: true },
  bad: { icon: AlertTriangle, box: "bg-warning-wash text-warning" },
};

function Row({ tone, title, hint }: { tone: Tone; title: string; hint: string }) {
  const { icon: Icon, box, spin } = TONES[tone];
  return (
    <div className="flex items-center gap-3 p-2 px-4">
      <div className={`grid size-9 shrink-0 place-items-center rounded-lg ${box}`}>
        <Icon size={18} className={spin ? "animate-spin" : ""} />
      </div>
      <div className="min-w-0">
        <h3 className="text-sm font-medium">{title}</h3>
        <p className="text-sm text-ink-2">{hint}</p>
      </div>
    </div>
  );
}

function pspDriverRow(status: SetupStatus) {
  switch (status.pspDriver) {
    case "ready":
      return (
        <Row
          tone="good"
          title="USB driver for the PSP"
          hint="Windows can talk to the PSP."
        />
      );
    case "needs-driver":
      return (
        <Row
          tone="bad"
          title="USB driver for the PSP"
          hint="Windows has no driver for the PSP yet. See the project page for what to do."
        />
      );
    case "not-connected":
      return (
        <Row
          tone="wait"
          title="USB driver for the PSP"
          hint="Checked once the program is running on the PSP."
        />
      );
  }
}

/** The card for the PSP step: it says what to do next, and does it when asked. */
function PspStep({
  status,
  installing,
  result,
  error,
  onInstall,
}: {
  status: SetupStatus;
  installing: boolean;
  result: PspInstall | null;
  error: string | null;
  onInstall: () => void;
}) {
  let body: ReactNode;
  if (!status.memoryStick) {
    body = (
      <Row
        tone="wait"
        title="Connect your PSP"
        hint="Plug it into this PC with its USB cable. On the PSP open Settings and choose USB Connection. This page notices it by itself."
      />
    );
  } else {
    body = (
      <div className="flex items-center gap-3 p-2 px-4">
        <div className="min-w-0 flex-1">
          <h3 className="text-sm font-medium">
            {status.programOnStick
              ? "PSP Sidescreen is on your PSP"
              : "Your PSP is connected"}
          </h3>
          <p className="text-sm text-ink-2">
            {status.programOnStick
              ? `It is on ${status.memoryStick}. Put it there again to update it.`
              : `It goes in Game, Memory Stick, on ${status.memoryStick}. Nothing else on the card is touched.`}
          </p>
        </div>
        <Button
          variant={status.programOnStick ? "secondary" : "primary"}
          disabled={installing}
          onClick={onInstall}
        >
          {installing
            ? "Copying…"
            : status.programOnStick
              ? "Update"
              : "Put it on my PSP"}
        </Button>
      </div>
    );
  }
  return (
    <>
      {body}
      {result && (
        <Row
          tone="good"
          title="Done"
          hint="On the PSP press Circle to leave USB Connection, then open PSP Sidescreen from Game, Memory Stick. It starts by itself."
        />
      )}
      {error && <Row tone="bad" title="That did not work" hint={error} />}
    </>
  );
}

export function SetupPage({
  sidescreen,
  onFinish,
}: {
  sidescreen: Sidescreen;
  onFinish: () => void;
}) {
  const { getSetup, installPspProgram } = sidescreen;
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [installing, setInstalling] = useState(false);
  const [result, setResult] = useState<PspInstall | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const look = () =>
      getSetup().then(
        (next) => !cancelled && setStatus(next),
        () => undefined,
      );
    void look();
    const timer = window.setInterval(() => void look(), LOOK_EVERY_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [getSetup]);

  const install = useCallback(() => {
    setInstalling(true);
    setError(null);
    setResult(null);
    installPspProgram()
      .then(setResult, (reason: unknown) => setError(String(reason)))
      .finally(() => setInstalling(false));
  }, [installPspProgram]);

  return (
    <div className="mx-auto w-full max-w-3xl space-y-6">
      <section className="rounded-lg border border-line bg-background p-4">
        <h2 className="text-base font-semibold">Set up PSP Sidescreen</h2>
        <p className="mt-1 text-sm text-ink-2">
          Two things on this PC, and the program on your PSP. The installer did the first
          two.
        </p>
      </section>

      <SettingsGroup title="On this PC">
        {status ? (
          <>
            <Row
              tone={status.virtualDisplay ? "good" : "bad"}
              title="Virtual display driver"
              hint={
                status.virtualDisplay
                  ? "Gives the PSP a screen of its own."
                  : "Missing. Install PSP Sidescreen again to add it. Without it the PSP can only show a monitor you already have."
              }
            />
            {pspDriverRow(status)}
          </>
        ) : (
          <Row tone="wait" title="Looking" hint="Checking this PC." />
        )}
      </SettingsGroup>

      <SettingsGroup title="On your PSP">
        {status ? (
          <PspStep
            status={status}
            installing={installing}
            result={result}
            error={error}
            onInstall={install}
          />
        ) : (
          <Row tone="wait" title="Looking" hint="Checking for a PSP." />
        )}
      </SettingsGroup>

      <div className="flex items-center justify-between px-1">
        <p className="flex items-center gap-2 text-sm text-ink-2">
          <Cable size={16} /> You can come back to this page at any time.
        </p>
        <Button size="lg" onClick={onFinish}>
          Done
        </Button>
      </div>
    </div>
  );
}
