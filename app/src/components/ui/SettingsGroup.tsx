import type { ReactNode } from "react";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  children: ReactNode;
}

/** A titled box of settings rows, like the groups of Handy's settings. */
export function SettingsGroup({ title, description, children }: SettingsGroupProps) {
  return (
    <section className="space-y-2">
      {title && (
        <div className="px-4">
          <h2 className="text-xs font-medium uppercase tracking-wide text-ink-3">
            {title}
          </h2>
          {description && <p className="mt-1 text-xs text-ink-3">{description}</p>}
        </div>
      )}
      <div className="overflow-visible rounded-lg border border-line bg-background">
        <div className="divide-y divide-line">{children}</div>
      </div>
    </section>
  );
}
