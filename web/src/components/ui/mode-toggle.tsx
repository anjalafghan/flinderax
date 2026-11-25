import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/context/ThemeContext";

/** Sun by day, moon by night: a little sky that flips when pressed. */
export function ModeToggle() {
  const { theme, setTheme } = useTheme();

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={() => setTheme(theme === "light" ? "dark" : "light")}
      aria-label={
        theme === "light" ? "Switch to dark theme" : "Switch to light theme"
      }
      className="relative h-11 w-11 overflow-hidden rounded-full border border-sun/40 bg-gradient-to-br from-sun/25 via-coral/15 to-sky/20 shadow-[0_6px_18px_-8px_hsl(var(--c-sun)/0.8)] transition-all duration-300 hover:scale-105 hover:bg-transparent dark:border-grape/40 dark:from-grape/30 dark:via-sky/15 dark:to-berry/25 dark:shadow-[0_6px_18px_-8px_hsl(var(--c-grape)/0.8)]"
    >
      <Moon className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 text-grape transition-all duration-500 dark:-rotate-90 dark:scale-0" />
      <Sun className="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 text-sun transition-all duration-500 dark:rotate-0 dark:scale-100" />
      <span className="sr-only">Toggle theme</span>
    </Button>
  );
}
