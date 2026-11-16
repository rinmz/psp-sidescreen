import { ChevronDown } from "lucide-react";
import type { SelectHTMLAttributes } from "react";

export interface Option<T extends string | number> {
  value: T;
  label: string;
}

interface SelectProps<T extends string | number> extends Omit<
  SelectHTMLAttributes<HTMLSelectElement>,
  "value" | "onChange"
> {
  value: T;
  options: readonly Option<T>[];
  onChange: (value: T) => void;
}

/** A native drop-down, so the keyboard and screen readers work as they do everywhere else. */
export function Select<T extends string | number>({
  value,
  options,
  onChange,
  className = "",
  ...props
}: SelectProps<T>) {
  return (
    <div className={`relative inline-block ${className}`}>
      <select
        value={value}
        onChange={(event) => {
          const chosen = options.find(
            (option) => String(option.value) === event.target.value,
          );
          if (chosen) onChange(chosen.value);
        }}
        className="appearance-none w-full min-w-44 cursor-pointer rounded-lg border border-line-strong bg-background py-[5px] ps-3 pe-9 text-sm text-text transition-colors hover:border-brand focus:outline-none focus-visible:ring-2 focus-visible:ring-brand disabled:cursor-not-allowed disabled:opacity-50"
        {...props}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <ChevronDown
        size={16}
        className="pointer-events-none absolute end-3 top-1/2 -translate-y-1/2 text-ink-3"
      />
    </div>
  );
}
