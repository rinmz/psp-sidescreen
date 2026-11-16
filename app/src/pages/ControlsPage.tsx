import { ShortcutRecorder } from "../components/ShortcutRecorder";
import { Select, type Option } from "../components/ui/Select";
import { SettingContainer } from "../components/ui/SettingContainer";
import { SettingsGroup } from "../components/ui/SettingsGroup";
import { Switch } from "../components/ui/Switch";
import type { Sidescreen } from "../hooks/useSidescreen";
import { CUSTOM, chordOf, isCustom } from "../lib/chord";

const NOTHING = "nothing";
/** The menu entry for a shortcut the person records; it is not an action of the program. */
const CUSTOM_CHOICE = "custom";

export function ControlsPage({ sidescreen }: { sidescreen: Sidescreen }) {
  const { settings, catalog, state, update } = sidescreen;
  const { controls } = settings;
  // The controls are read when a session starts, so they are changed with it stopped.
  const locked = state.kind !== "idle" && state.kind !== "error";
  const off = locked || !controls.enabled;

  const options: Option<string>[] = [
    ...catalog.actions.map((action) => ({ value: action.id, label: action.label })),
    { value: CUSTOM_CHOICE, label: "Custom shortcut…" },
  ];

  const change = (change: Partial<typeof controls>) =>
    update({ controls: { ...controls, ...change } });

  const setButton = (button: string, setting: string) =>
    change({ buttons: { ...controls.buttons, [button]: setting } });

  return (
    <div className="mx-auto w-full max-w-3xl space-y-6">
      <SettingsGroup
        title="PSP controls"
        description={locked ? "Stop to change these." : undefined}
      >
        <SettingContainer
          title="Use the PSP's buttons"
          description="Lets the PSP's buttons press keys and click for you on this PC. Nothing is sent until you turn this on."
          disabled={locked}
        >
          <Switch
            label="Use the PSP's buttons"
            checked={controls.enabled}
            onChange={(enabled) => change({ enabled })}
            disabled={locked}
          />
        </SettingContainer>
        <SettingContainer
          title="Stick moves the mouse"
          description="Pushing the analog stick moves the mouse pointer. A small push is slow and fine, a full push is fast."
          disabled={off}
        >
          <Switch
            label="Stick moves the mouse"
            checked={controls.analogMouse}
            onChange={(analogMouse) => change({ analogMouse })}
            disabled={off}
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup
        title="Buttons"
        description="A shortcut stays pressed while you hold the button, like holding the keys."
      >
        {catalog.buttons.map((button) => {
          const setting = controls.buttons[button.id] ?? NOTHING;
          const custom = isCustom(setting);
          return (
            <SettingContainer
              key={button.id}
              title={button.label}
              description={`What the ${button.label} button does on the PC.`}
              disabled={off}
            >
              <div className="flex items-center gap-2">
                {custom && (
                  <ShortcutRecorder
                    label={button.label}
                    chord={chordOf(setting)}
                    onChange={(chord) => setButton(button.id, `${CUSTOM}${chord}`)}
                    disabled={off}
                  />
                )}
                <Select
                  aria-label={button.label}
                  value={custom ? CUSTOM_CHOICE : setting}
                  options={options}
                  onChange={(choice) =>
                    setButton(button.id, choice === CUSTOM_CHOICE ? CUSTOM : choice)
                  }
                  disabled={off}
                />
              </div>
            </SettingContainer>
          );
        })}
      </SettingsGroup>
    </div>
  );
}
