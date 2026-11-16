import { StatusCard } from "../components/StatusCard";
import { Select, type Option } from "../components/ui/Select";
import { SettingContainer } from "../components/ui/SettingContainer";
import { SettingsGroup } from "../components/ui/SettingsGroup";
import { SIZES, type Mode, type Quality } from "../lib/backend";
import type { Sidescreen } from "../hooks/useSidescreen";

const MODES: Option<Mode>[] = [
  { value: "extended", label: "Extended screen" },
  { value: "mirror", label: "Mirror a monitor" },
];

const QUALITIES: Option<Quality>[] = [
  { value: "lossless", label: "Lossless" },
  { value: "high", label: "High" },
  { value: "medium", label: "Medium" },
  { value: "low", label: "Low" },
];

const SIZE_OPTIONS: Option<string>[] = SIZES.map((size) => ({
  value: size,
  label: `${size.replace("x", " × ")}${size === "1366x768" ? " (recommended)" : ""}`,
}));

export function DisplayPage({ sidescreen }: { sidescreen: Sidescreen }) {
  const { settings, monitors, state, update, start, stop } = sidescreen;
  // The settings are read when a session starts, so they are changed with it stopped.
  const locked = state.kind !== "idle" && state.kind !== "error";

  const monitorOptions: Option<number>[] = monitors.map((monitor) => ({
    value: monitor.index,
    label: `${monitor.index + 1}: ${monitor.width} × ${monitor.height}`,
  }));

  return (
    <div className="mx-auto w-full max-w-3xl space-y-6">
      <StatusCard state={state} onStart={start} onStop={stop} />

      <SettingsGroup
        title="Screen"
        description={locked ? "Stop to change these." : undefined}
      >
        <SettingContainer
          title="What the PSP shows"
          description="Extended gives the PSP a screen of its own that you drag windows onto. Mirror shows a monitor you already have."
          disabled={locked}
        >
          <Select
            aria-label="What the PSP shows"
            value={settings.mode}
            options={MODES}
            onChange={(mode) => update({ mode })}
            disabled={locked}
          />
        </SettingContainer>
        {settings.mode === "extended" ? (
          <SettingContainer
            title="Screen size"
            description="The PSP's screen is 480 × 272, so a larger size gives you more room, not sharper text. For sharper text keep 1366 × 768 and raise the Windows display scaling."
            disabled={locked}
          >
            <Select
              aria-label="Screen size"
              value={settings.size}
              options={SIZE_OPTIONS}
              onChange={(size) => update({ size })}
              disabled={locked}
            />
          </SettingContainer>
        ) : (
          <SettingContainer
            title="Monitor"
            description="The monitor whose picture the PSP shows."
            disabled={locked}
          >
            <Select
              aria-label="Monitor"
              value={settings.monitor}
              options={monitorOptions}
              onChange={(monitor) => update({ monitor })}
              disabled={locked}
            />
          </SettingContainer>
        )}
      </SettingsGroup>

      <SettingsGroup title="Picture">
        <SettingContainer
          title="Quality"
          description="Lossless sends every pixel and looks sharpest. High, Medium and Low compress the picture more, which sends smaller frames."
          disabled={locked}
        >
          <Select
            aria-label="Quality"
            value={settings.quality}
            options={QUALITIES}
            onChange={(quality) => update({ quality })}
            disabled={locked}
          />
        </SettingContainer>
      </SettingsGroup>
    </div>
  );
}
