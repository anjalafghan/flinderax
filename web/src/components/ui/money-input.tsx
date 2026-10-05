import * as React from "react";
import { Input } from "@/components/ui/input";

/** Rupee text input. Parse its value with `toPaise` from `@/utils/money`. */
export const MoneyInput = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  (props, ref) => <Input ref={ref} inputMode="decimal" placeholder="0" autoComplete="off" {...props} />,
);
MoneyInput.displayName = "MoneyInput";
