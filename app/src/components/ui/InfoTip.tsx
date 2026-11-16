import { Info } from "lucide-react";

/** A small "i" that explains a setting when hovered or focused. */
export function InfoTip({ text }: { text: string }) {
  return (
    <span className="group relative inline-flex">
      <button
        type="button"
        aria-label={text}
        className="rounded-full text-ink-3 transition-colors hover:text-text focus:outline-none focus-visible:ring-2 focus-visible:ring-brand"
      >
        <Info size={15} />
      </button>
      <span
        role="tooltip"
        className="pointer-events-none absolute bottom-full start-1/2 z-10 mb-2 w-60 -translate-x-1/2 rounded-lg border border-line bg-background p-2 text-xs leading-snug text-text opacity-0 shadow-lg transition-opacity group-hover:opacity-100 group-focus-within:opacity-100"
      >
        {text}
      </span>
    </span>
  );
}
