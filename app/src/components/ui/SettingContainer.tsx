import type { ReactNode } from "react";

import { InfoTip } from "./InfoTip";

interface SettingContainerProps {
  title: string;
  /** Shown in a tooltip beside the title. */
  description: string;
  disabled?: boolean;
  children: ReactNode;
}

/** One row of a settings group: the title on the left, the control on the right. */
export function SettingContainer({
  title,
  description,
  disabled = false,
  children,
}: SettingContainerProps) {
  return (
    <div className="flex min-h-12 items-center justify-between gap-4 p-2 px-4">
      <div className="flex max-w-2/3 items-center gap-2">
        <h3 className={`text-sm font-medium ${disabled ? "opacity-50" : ""}`}>{title}</h3>
        <InfoTip text={description} />
      </div>
      <div className="relative">{children}</div>
    </div>
  );
}
