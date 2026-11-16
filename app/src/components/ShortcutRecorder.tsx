import { useEffect, useRef, useState } from "react";

import { chordFromKeys, formatChord, isModifier } from "../lib/chord";

interface ShortcutRecorderProps {
  /** The shortcut, as `ControlLeft+Space`; empty when there is none yet. */
  chord: string;
  onChange: (chord: string) => void;
  disabled?: boolean;
  /** Names the field for assistive technology, such as the button it belongs to. */
  label: string;
}

/**
 * Shows a shortcut and records a new one: press the button, then the keys together. It ends on
 * the first key that is not a modifier, or when only modifiers were pressed and let go.
 * Escape on its own cancels.
 */
export function ShortcutRecorder({
  chord,
  onChange,
  disabled = false,
  label,
}: ShortcutRecorderProps) {
  const [recording, setRecording] = useState(false);
  // Keys held now, and keys seen since the recording began, both in the order they were pressed.
  const held = useRef<string[]>([]);
  const seen = useRef<string[]>([]);

  useEffect(() => {
    if (!recording) return;
    held.current = [];
    seen.current = [];

    const finish = (next: string | null) => {
      setRecording(false);
      if (next) onChange(next);
    };

    const onKeyDown = (event: KeyboardEvent) => {
      // The page must not act on the keys while they are being recorded.
      event.preventDefault();
      event.stopPropagation();
      if (event.repeat) return;
      if (event.code === "Escape" && held.current.length === 0) return finish(null);
      held.current.push(event.code);
      seen.current.push(event.code);
      if (!isModifier(event.code)) finish(chordFromKeys(seen.current));
    };

    const onKeyUp = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      held.current = held.current.filter((code) => code !== event.code);
      // Only modifiers were pressed, and now all are let go: that is the shortcut.
      if (held.current.length === 0 && seen.current.length > 0) {
        finish(chordFromKeys(seen.current));
      }
    };

    const stop = () => setRecording(false);

    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    window.addEventListener("blur", stop);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
      window.removeEventListener("blur", stop);
    };
  }, [recording, onChange]);

  const text = recording
    ? "Press the keys…"
    : chord === ""
      ? "Record a shortcut"
      : formatChord(chord);

  return (
    <button
      type="button"
      aria-label={`${label}: ${chord === "" ? "no shortcut" : formatChord(chord)}. Press to record a new one.`}
      aria-pressed={recording}
      disabled={disabled}
      onClick={() => setRecording((current) => !current)}
      className={`min-w-44 cursor-pointer rounded-lg border px-3 py-[5px] text-sm transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-brand disabled:cursor-not-allowed disabled:opacity-50 ${
        recording
          ? "border-brand bg-brand/20 text-text"
          : "border-line-strong bg-background text-text hover:border-brand"
      } ${chord === "" && !recording ? "text-ink-2" : ""}`}
    >
      {text}
    </button>
  );
}
