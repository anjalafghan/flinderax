// A see-through, aria-hidden layer over the page for the hearts that fly off the stamp. It lives
// inside the .joy tree (never on <body>) so it keeps the page's colours and scoping.
import { useEffect, useImperativeHandle, useRef, useState, type Ref } from "react";
import { animated, easings, to, useSpring } from "@react-spring/web";
import { addHearts, HEART_TTL, makeHearts, pruneHearts, type HeartBit } from "./hearts";
import { reduced } from "./motion";

export interface HeartHandle {
  /** Throw 2-3 hearts from (x, y), in viewport pixels. Nothing flies under reduced motion. */
  burst: (x: number, y: number) => void;
}

const HEART = "M8 14.2C4.6 11.6 1.7 9.2 1.8 6.1c.1-2.2 1.7-3.6 3.5-3.5 1.2 0 2.1.7 2.7 1.6.6-.9 1.6-1.7 2.9-1.6 1.9.1 3.4 1.6 3.3 3.7-.1 3-3.1 5.4-6.2 7.9z";

function Heart({ h }: { h: HeartBit }) {
  const [s] = useSpring(() => ({
    from: { x: 0, y: 0, r: 0, k: 0.2, o: 1 },
    to: async (next) => {
      await next({ x: h.dx, y: h.dy, r: h.rot * 0.4, k: 1, config: { tension: 300, friction: 20 } });
      await next({ x: h.dx * 1.15, y: h.dy + h.fall, r: h.rot, o: 0, config: { duration: 650, easing: easings.easeInQuad } });
    },
    delay: h.delay,
  }));
  return (
    <animated.span
      className="m-bit m-bit-hearts"
      style={{
        left: h.x,
        top: h.y,
        width: h.size,
        height: h.size,
        color: "var(--stamp)",
        opacity: s.o,
        transform: to([s.x, s.y, s.r, s.k], (x, y, r, k) => `translate(-50%, -50%) translate(${x}px, ${y}px) rotate(${r}deg) scale(${k})`),
      }}
    >
      <svg viewBox="0 0 16 16">
        <path d={HEART} />
      </svg>
    </animated.span>
  );
}

export default function HeartLayer({ handle }: { handle: Ref<HeartHandle> }) {
  const [hearts, setHearts] = useState<HeartBit[]>([]);
  const nextId = useRef(1);

  useImperativeHandle(handle, () => ({
    burst(x, y) {
      if (reduced()) return;
      const now = performance.now();
      const fresh = makeHearts(x, y, now, nextId.current);
      nextId.current += fresh.length;
      setHearts((l) => addHearts(pruneHearts(l, now), fresh));
    },
  }), []);

  // Clear them once they've fallen.
  useEffect(() => {
    if (!hearts.length) return;
    const t = setTimeout(() => setHearts((l) => pruneHearts(l, performance.now())), HEART_TTL + 20);
    return () => clearTimeout(t);
  }, [hearts]);

  return (
    <div className="m-layer" aria-hidden="true">
      {hearts.map((h) => (
        <Heart key={h.id} h={h} />
      ))}
    </div>
  );
}
