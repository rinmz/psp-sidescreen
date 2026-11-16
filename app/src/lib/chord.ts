/**
 * Shortcuts the person records for a PSP button.
 *
 * A shortcut is kept as the names a web page gives keys (`KeyboardEvent.code`) joined by `+`,
 * such as `ControlLeft+Space`. The program behind the window decides which names it understands;
 * this file only collects them as keys are pressed and writes them for a person to read.
 */

/** Settings that hold a recorded shortcut start with this, then the shortcut. */
export const CUSTOM = "custom:";

const MODIFIER_PREFIXES = ["Control", "Shift", "Alt", "Meta"] as const;

export function isCustom(setting: string): boolean {
  return setting.startsWith(CUSTOM);
}

/** The shortcut inside a custom setting, or an empty string if there is none yet. */
export function chordOf(setting: string): string {
  return isCustom(setting) ? setting.slice(CUSTOM.length) : "";
}

export function isModifier(code: string): boolean {
  return MODIFIER_PREFIXES.some((prefix) => code.startsWith(prefix));
}

/**
 * The shortcut for the keys held down: the modifiers in the order they were pressed, then the
 * other key, if there is one.
 */
export function chordFromKeys(held: readonly string[]): string {
  const modifiers = held.filter(isModifier);
  const keys = held.filter((code) => !isModifier(code));
  return [...modifiers, ...keys].join("+");
}

const NAMES: Record<string, string> = {
  Space: "Space",
  Enter: "Enter",
  Escape: "Esc",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  PageUp: "Page Up",
  PageDown: "Page Down",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  AudioVolumeUp: "Volume up",
  AudioVolumeDown: "Volume down",
  AudioVolumeMute: "Mute",
  MediaPlayPause: "Play/Pause",
  MediaTrackNext: "Next track",
  MediaTrackPrevious: "Previous track",
};

/** How a key is shown to a person. */
export function keyLabel(code: string): string {
  if (code.startsWith("Control")) return "Ctrl";
  if (code.startsWith("Shift")) return "Shift";
  if (code.startsWith("Alt")) return "Alt";
  if (code.startsWith("Meta")) return "Win";
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  return NAMES[code] ?? code;
}

/** A shortcut as a person reads it, such as `Ctrl + Space`. */
export function formatChord(chord: string): string {
  return chord
    .split("+")
    .filter((code) => code !== "")
    .map(keyLabel)
    .join(" + ");
}
