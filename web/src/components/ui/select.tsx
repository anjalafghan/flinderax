import * as React from "react";
import { cn } from "@/utils/cn";

const Select = React.forwardRef<
  HTMLSelectElement,
  React.SelectHTMLAttributes<HTMLSelectElement>
>(({ className, ...props }, ref) => (
  <select
    ref={ref}
    className={cn(
      "flex h-11 w-full rounded-xl border border-input bg-card px-3.5 py-2 text-sm ring-offset-background transition-[border-color,box-shadow] hover:border-brand/40 focus-visible:border-brand focus-visible:shadow-[0_0_0_4px_hsl(var(--brand)/0.15)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
      className,
    )}
    {...props}
  />
));
Select.displayName = "Select";

export { Select };
