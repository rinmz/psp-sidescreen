import {
  AlertTriangle,
  Cable,
  CheckCircle2,
  Loader2,
  MonitorPlay,
  RefreshCw,
  type LucideIcon,
} from "lucide-react";

import type { SessionState } from "../lib/backend";
import { Button } from "./ui/Button";

type Tone = "neutral" | "good" | "warn" | "bad";

interface View {
  icon: LucideIcon;
  tone: Tone;
  title: string;
  hint: string;
  spin?: boolean;
}

/** What each state of the session says to the person. */
export function describe(state: SessionState): View {
  switch (state.kind) {
    case "idle":
      return {
        icon: MonitorPlay,
        tone: "neutral",
        title: "Ready",
        hint: "Open Sidescreen on your PSP, then press Start.",
      };
    case "starting":
      return {
        icon: Loader2,
        tone: "neutral",
        title: "Starting",
        hint: "Setting up the screen for your PSP.",
        spin: true,
      };
    case "waiting":
      return {
        icon: Cable,
        tone: "warn",
        title: "Looking for your PSP",
        hint: "Connect it with the USB cable, open Sidescreen on it and choose Start using USB.",
      };
    case "streaming":
      return {
        icon: CheckCircle2,
        tone: "good",
        title: "Showing on your PSP",
        hint:
          state.fps === 0
            ? `Sending the first picture, ${state.width}×${state.height}.`
            : `${state.fps} frames per second, ${state.width}×${state.height}.`,
      };
    case "reconnecting":
      return {
        icon: RefreshCw,
        tone: "warn",
        title: "Picking the screen up again",
        hint: "The monitor changed. This takes a moment.",
        spin: true,
      };
    case "error":
      return {
        icon: AlertTriangle,
        tone: "bad",
        title: "Something went wrong",
        hint: state.message,
      };
  }
}

const TONES: Record<Tone, string> = {
  neutral: "bg-wash text-text",
  good: "bg-moss-wash text-moss",
  warn: "bg-warning-wash text-warning",
  bad: "bg-error-wash text-error",
};

interface StatusCardProps {
  state: SessionState;
  onStart: () => void;
  onStop: () => void;
}

export function StatusCard({ state, onStart, onStop }: StatusCardProps) {
  const { icon: Icon, tone, title, hint, spin } = describe(state);
  const running = state.kind !== "idle" && state.kind !== "error";

  return (
    <section
      aria-live="polite"
      className="flex animate-rise items-center gap-4 rounded-lg border border-line bg-background p-4"
    >
      <div
        className={`grid size-12 shrink-0 place-items-center rounded-xl ${TONES[tone]}`}
      >
        <Icon size={24} className={spin ? "animate-spin" : ""} />
      </div>
      <div className="min-w-0 flex-1">
        <h2 className="text-base font-semibold">{title}</h2>
        <p className="text-sm text-ink-2">{hint}</p>
      </div>
      {running ? (
        <Button variant="secondary" size="lg" onClick={onStop}>
          Stop
        </Button>
      ) : (
        <Button size="lg" onClick={onStart}>
          Start
        </Button>
      )}
    </section>
  );
}
