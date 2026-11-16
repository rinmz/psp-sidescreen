import { describe, expect, it } from "vitest";

import {
  CUSTOM,
  chordFromKeys,
  chordOf,
  formatChord,
  isCustom,
  isModifier,
  keyLabel,
} from "./chord";

describe("settings that hold a shortcut", () => {
  it("are recognised by their prefix", () => {
    expect(isCustom(`${CUSTOM}ControlLeft+Space`)).toBe(true);
    expect(isCustom("enter")).toBe(false);
    expect(isCustom("")).toBe(false);
  });

  it("give back the shortcut inside them", () => {
    expect(chordOf("custom:ControlLeft+Space")).toBe("ControlLeft+Space");
    expect(chordOf("custom:")).toBe("");
  });

  it("have no shortcut when they are a preset", () => {
    expect(chordOf("enter")).toBe("");
  });
});

describe("recording keys", () => {
  it("tells modifiers from other keys", () => {
    for (const code of [
      "ControlLeft",
      "ControlRight",
      "ShiftLeft",
      "AltRight",
      "MetaLeft",
    ]) {
      expect(isModifier(code), code).toBe(true);
    }
    for (const code of ["Space", "KeyA", "Digit1", "F5", "ArrowUp"]) {
      expect(isModifier(code), code).toBe(false);
    }
  });

  it("puts the modifiers first, in the order they were pressed", () => {
    expect(chordFromKeys(["ShiftLeft", "ControlLeft", "KeyH"])).toBe(
      "ShiftLeft+ControlLeft+KeyH",
    );
  });

  it("puts a key pressed before a modifier after it", () => {
    expect(chordFromKeys(["Space", "ControlLeft"])).toBe("ControlLeft+Space");
  });

  it("is empty when nothing is held", () => {
    expect(chordFromKeys([])).toBe("");
  });

  it("can be modifiers alone", () => {
    expect(chordFromKeys(["ControlLeft", "AltLeft"])).toBe("ControlLeft+AltLeft");
  });
});

describe("showing a shortcut", () => {
  it("writes Handy's shortcut the way a person would", () => {
    expect(formatChord("ControlLeft+Space")).toBe("Ctrl + Space");
  });

  it("does not tell left from right", () => {
    expect(keyLabel("ControlRight")).toBe("Ctrl");
    expect(keyLabel("ShiftLeft")).toBe("Shift");
    expect(keyLabel("MetaRight")).toBe("Win");
  });

  it("shows letters and digits without their prefix", () => {
    expect(keyLabel("KeyH")).toBe("H");
    expect(keyLabel("Digit7")).toBe("7");
  });

  it("shows other keys by a readable name, or as they are", () => {
    expect(keyLabel("ArrowUp")).toBe("Up");
    expect(keyLabel("F12")).toBe("F12");
    expect(keyLabel("SomethingNew")).toBe("SomethingNew");
  });

  it("is empty for no shortcut", () => {
    expect(formatChord("")).toBe("");
  });
});
