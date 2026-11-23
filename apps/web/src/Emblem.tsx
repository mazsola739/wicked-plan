// The Observer: Wicked Plan's emblem (docs/design.md, Visual identity). The art is the user's own:
// assets/observer.png (the emblem with crown and ring), assets/hood.webp (the hooded Observer from
// the poster) and assets/face.webp (its LED eyes and grin alone). The small crown mark is SVG.

import { useEffect, useState } from 'react';
import face from './assets/face.webp';
import observer from './assets/observer.png';
import './Emblem.css';

const ALT = 'The Observer, a hooded figure with a glowing face';

/** True when the viewer asked the system for less motion. */
export function reducedMotion(): boolean {
  return typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * The hooded Observer: by default the emblem with its crown and ring, or other art of it (`src`).
 * With `watching`, its signal glitches now and then (every 7–16 s: a cyan/violet split and a
 * flash), as if it were watching you on a live feed.
 */
export function Emblem({
  className = '',
  src = observer,
  watching = false,
  decorative = false,
}: {
  className?: string;
  src?: string;
  watching?: boolean;
  /** Pure decoration (the game bar, the transmission monitor): hidden from screen readers. */
  decorative?: boolean;
}) {
  const [glitch, setGlitch] = useState(false);
  useEffect(() => {
    if (!watching || reducedMotion()) return;
    let timer = 0;
    const next = () => {
      timer = window.setTimeout(
        () => {
          setGlitch(true);
          timer = window.setTimeout(() => {
            setGlitch(false);
            next();
          }, 700);
        },
        7000 + Math.random() * 9000,
      );
    };
    next();
    return () => window.clearTimeout(timer);
  }, [watching]);

  if (!watching) return <img className={`emblem ${className}`} src={src} alt={decorative ? '' : ALT} />;
  const label = decorative ? { 'aria-hidden': true } : { role: 'img', 'aria-label': ALT };
  return (
    <span className={`emblem-watch ${className}${glitch ? ' glitch' : ''}`} {...label}>
      <img className="emblem" src={src} alt="" />
      <img className="emblem em-ghost em-cyan" src={src} alt="" />
      <img className="emblem em-ghost em-violet" src={src} alt="" />
    </span>
  );
}

/**
 * Only the Observer's face, without the hood, peeking at you during a game (docs/design.md,
 * Broadcast feel): its LED eyes and grin (assets/face.webp). Mostly invisible; every 9–20 s its
 * gaze shines through the dark: it surfaces blurred, sharpens, flickers, glances aside and fades.
 * Nothing shows with reduced motion.
 */
export function Peeker({ className = '' }: { className?: string }) {
  const [peek, setPeek] = useState(false);
  useEffect(() => {
    if (reducedMotion()) return;
    let timer = 0;
    const schedule = (wait: number) => {
      timer = window.setTimeout(() => {
        setPeek(true);
        timer = window.setTimeout(() => {
          setPeek(false);
          schedule(9000 + Math.random() * 11000);
        }, PEEK_MS);
      }, wait);
    };
    schedule(2500 + Math.random() * 3000);
    return () => window.clearTimeout(timer);
  }, []);
  return (
    <span className={`peeker ${className}${peek ? ' peek' : ''}`} aria-hidden="true">
      <img src={face} alt="" />
    </span>
  );
}

const PEEK_MS = 3400;

/** The crown alone: the small brand mark. */
export function Crown({ className = '' }: { className?: string }) {
  return (
    <svg className={`crown ${className}`} viewBox="10 -4 80 40" aria-hidden="true">
      <path
        d="M14,32 L18,15 L30,25 L36,7 L44,21 L50,-1 L56,21 L64,7 L70,25 L82,15 L86,32 Q50,26 14,32 Z"
        fill="none"
        stroke="currentColor"
        strokeWidth={3}
        strokeLinejoin="miter"
      />
    </svg>
  );
}
