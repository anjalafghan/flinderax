import { useEffect, type ReactNode } from "react";
import ClickSpark from "@/components/bits/ClickSpark";
import { useSection } from "./useSection";

/** Marks the current tab on <html data-section> and throws a little burst of ink sparks wherever you click. */
export function SectionTheme({ children }: { children: ReactNode }) {
  const { section, colors } = useSection();

  useEffect(() => {
    document.documentElement.dataset.section = section;
  }, [section]);

  return (
    <ClickSpark sparkColor={colors[1]} sparkCount={10} sparkRadius={26} sparkSize={12} shape="star" duration={520} className="!h-auto">
      {children}
    </ClickSpark>
  );
}
