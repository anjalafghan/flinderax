import { useLocation } from "react-router-dom";
import { useTheme } from "@/context/ThemeContext";
import { SECTION_COLORS, sectionFor, type Section } from "./sections";

/** The current tab's section and its three colours for the current light/dark theme. */
export function useSection(): { section: Section; colors: [string, string, string]; dark: boolean } {
  const { pathname } = useLocation();
  const { theme } = useTheme();
  const section = sectionFor(pathname);
  const dark = theme === "dark";
  return { section, colors: SECTION_COLORS[section][dark ? "dark" : "light"], dark };
}
