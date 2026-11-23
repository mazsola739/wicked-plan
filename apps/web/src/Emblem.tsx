// The Observer: Wicked Plan's emblem (docs/design.md, Visual identity). The emblem is the user's
// own artwork (assets/observer.png); the small crown mark is drawn here in SVG.

import { useEffect, useState } from 'react';
import observer from './assets/observer.png';
import './Emblem.css';

const ALT = 'The Observer, a hooded figure with a glowing face';

/** True when the viewer asked the system for less motion. */
export function reducedMotion(): boolean {
  return typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * The hooded Observer with its crown and ring. With `watching`, its signal glitches now and then
 * (every 7–16 s: a cyan/violet split and a flash), as if it were watching you on a live feed.
 */
export function Emblem({
  className = '',
  watching = false,
  decorative = false,
}: {
  className?: string;
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

  if (!watching) return <img className={`emblem ${className}`} src={observer} alt={decorative ? '' : ALT} />;
  const label = decorative ? { 'aria-hidden': true } : { role: 'img', 'aria-label': ALT };
  return (
    <span className={`emblem-watch ${className}${glitch ? ' glitch' : ''}`} {...label}>
      <img className="emblem" src={observer} alt="" />
      <img className="emblem em-ghost em-cyan" src={observer} alt="" />
      <img className="emblem em-ghost em-violet" src={observer} alt="" />
    </span>
  );
}

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
