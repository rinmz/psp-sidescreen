import { Gamepad2, Info, Monitor, type LucideIcon } from "lucide-react";
import type { CSSProperties } from "react";

import badgeUrl from "../assets/logo-badge.png";
import markUrl from "../assets/logo-mark.png";

export type Section = "display" | "controls" | "about";

const SECTIONS: { id: Section; label: string; icon: LucideIcon }[] = [
  { id: "display", label: "Display", icon: Monitor },
  { id: "controls", label: "Controls", icon: Gamepad2 },
  { id: "about", label: "About", icon: Info },
];

interface SidebarProps {
  active: Section;
  onChange: (section: Section) => void;
}

export function Sidebar({ active, onChange }: SidebarProps) {
  return (
    <nav
      aria-label="Sections"
      className="flex h-full w-40 shrink-0 flex-col items-center border-e border-line px-2"
    >
      <div className="m-4 flex flex-col items-center gap-2">
        <div
          role="img"
          aria-label="PSP"
          className="logo-mark w-[112px]"
          style={
            {
              "--logo-mark": `url(${markUrl})`,
              aspectRatio: "663 / 116",
            } as CSSProperties
          }
        />
        <img src={badgeUrl} alt="sidescreen" className="w-[104px]" draggable={false} />
      </div>
      <div className="flex w-full flex-col items-center gap-1 border-t border-line pt-2">
        {SECTIONS.map(({ id, label, icon: Icon }) => {
          const isActive = id === active;
          return (
            <button
              key={id}
              type="button"
              aria-current={isActive ? "page" : undefined}
              onClick={() => onChange(id)}
              className={`flex w-full cursor-pointer items-center gap-2 rounded-lg p-2 text-start transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-brand ${
                isActive
                  ? "bg-brand text-on-brand"
                  : "opacity-85 hover:bg-line hover:opacity-100"
              }`}
            >
              <Icon size={22} className="shrink-0" />
              <span className="truncate text-sm font-medium">{label}</span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}
