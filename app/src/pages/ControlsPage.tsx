import { Select, type Option } from "../components/ui/Select";
import { SettingContainer } from "../components/ui/SettingContainer";
import { SettingsGroup } from "../components/ui/SettingsGroup";
import { Switch } from "../components/ui/Switch";
import type { Sidescreen } from "../hooks/useSidescreen";

const NOTHING = "nothing";

export function ControlsPage({ sidescreen }: { sidescreen: Sidescreen }) {
  const { settings, catalog, state, update } = sidescreen;
  const { controls } = settings;
  // The controls are read when a session starts, so they are changed with it stopped.
  const locked = state.kind !== "idle" && state.kind !== "error";
  const off = locked || !controls.enabled;

  const actions: Option<string>[] = catalog.actions.map((action) => ({
    value: action.id,
    label: action.label,
  }));

  const change = (change: Partial<typeof controls>) =>
    update({ controls: { ...controls, ...change } });

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

      <SettingsGroup title="Buttons">
        {catalog.buttons.map((button) => (
          <SettingContainer
            key={button.id}
            title={button.label}
            description={`What the ${button.label} button does on the PC.`}
            disabled={off}
          >
            <Select
              aria-label={button.label}
              value={controls.buttons[button.id] ?? NOTHING}
              options={actions}
              onChange={(action) =>
                change({ buttons: { ...controls.buttons, [button.id]: action } })
              }
              disabled={off}
            />
          </SettingContainer>
        ))}
      </SettingsGroup>
    </div>
  );
}
