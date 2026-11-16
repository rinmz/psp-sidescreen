import type { ButtonHTMLAttributes } from "react";

type Variant = "primary" | "secondary" | "ghost";
type Size = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
}

const BASE =
  "font-medium rounded-lg border transition-colors cursor-pointer focus:outline-none " +
  "focus-visible:ring-2 focus-visible:ring-brand focus-visible:ring-offset-2 " +
  "focus-visible:ring-offset-background disabled:opacity-50 disabled:cursor-not-allowed";

const VARIANTS: Record<Variant, string> = {
  // The logo's yellow with the logo's ink on it; white would not read on yellow.
  primary: "text-on-brand bg-brand border-brand hover:brightness-95 active:brightness-90",
  secondary: "text-text bg-wash border-line hover:bg-brand/20 hover:border-brand",
  ghost: "text-current border-transparent hover:bg-wash hover:border-brand",
};

const SIZES: Record<Size, string> = {
  sm: "px-2 py-1 text-xs",
  md: "px-4 py-[5px] text-sm",
  lg: "px-5 py-2 text-base",
};

export function Button({
  variant = "primary",
  size = "md",
  className = "",
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      className={`${BASE} ${VARIANTS[variant]} ${SIZES[size]} ${className}`}
      {...props}
    />
  );
}
