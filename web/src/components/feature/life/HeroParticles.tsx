import Particles, {
  type ParticlesDrift,
  type ParticlesShape,
} from "@/components/bits/Particles";
import { useSection } from "@/theme/useSection";

/**
 * A slow drift of coloured specks behind a page title. Loaded lazily (it is WebGL), decorative only,
 * and still when the user prefers less motion (Particles checks that itself).
 */
export default function HeroParticles({
  colors,
  count = 90,
  shape = "circle",
  drift = "none",
}: {
  /** Hex colours; the section's own when left out. */
  colors?: string[];
  count?: number;
  shape?: ParticlesShape;
  drift?: ParticlesDrift;
}) {
  const { colors: section, dark } = useSection();
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute inset-0 -z-10 [mask-image:radial-gradient(ellipse_at_30%_40%,black_30%,transparent_75%)]"
    >
      <Particles
        particleColors={colors ?? section}
        particleCount={count}
        particleSpread={9}
        speed={0.06}
        particleBaseSize={70}
        sizeRandomness={0.8}
        alphaParticles
        disableRotation={false}
        lightMode={!dark}
        opacity={dark ? 0.85 : 0.7}
        pixelRatio={1}
        shape={shape}
        drift={drift}
        driftSpeed={0.4}
      />
    </div>
  );
}
