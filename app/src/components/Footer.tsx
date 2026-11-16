import type { SessionState } from "../lib/backend";
import { describe } from "./StatusCard";

const DOTS = {
  neutral: "bg-ink-3",
  good: "bg-moss",
  warn: "bg-warning",
  bad: "bg-error",
} as const;

interface FooterProps {
  state: SessionState;
  version: string;
}

/** The thin bar at the bottom: where the session is, and which version this is. */
export function Footer({ state, version }: FooterProps) {
  const { tone, title } = describe(state);
  return (
    <footer className="w-full border-t border-line pt-3">
      <div className="flex items-center justify-between px-4 pb-3 text-xs text-ink-2">
        <div className="flex items-center gap-2">
          <span className={`size-2 rounded-full ${DOTS[tone]}`} aria-hidden="true" />
          <span>{title}</span>
        </div>
        <span>v{version}</span>
      </div>
    </footer>
  );
}
