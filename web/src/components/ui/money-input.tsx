import * as React from "react";
import { Input } from "@/components/ui/input";
import { cn } from "@/utils/cn";

/** Rupee text input. Parse its value with `toPaise` from `@/utils/money`. */
export const MoneyInput = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className, ...props }, ref) => (
  <Input
    ref={ref}
    inputMode="decimal"
    placeholder="0"
    autoComplete="off"
    className={cn("font-medium tabular-nums", className)}
    {...props}
  />
));
MoneyInput.displayName = "MoneyInput";
