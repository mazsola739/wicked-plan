// The Observer: Wicked Plan's emblem (docs/design.md, Visual identity). The art is the owner's own:
// assets/hood.webp (the hooded Observer from the poster) and assets/face.webp (its LED eyes and
// grin alone). The small crown mark is SVG.

import { useEffect, useRef, useState } from 'react';
import face from './assets/face.webp';
import hoodArt from './assets/hood.webp';
import './Emblem.css';

const ALT = 'The Observer, a hooded figure with a glowing face';

/** True when the viewer asked the system for less motion. */
export function reducedMotion(): boolean {
  return typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * The hooded Observer: the large hooded figure from the poster. With `watching`, its signal glitches
 * now and then (every 7–16 s: a cyan/violet split and a flash), as if it were watching you on a live
 * feed.
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

  const src = hoodArt;
  const art = 'emblem';
  if (!watching) return <img className={`${art} ${className}`} src={src} alt={decorative ? '' : ALT} />;
  const label = decorative ? { 'aria-hidden': true } : { role: 'img', 'aria-label': ALT };
  return (
    <span className={`emblem-watch ${className}${glitch ? ' glitch' : ''}`} {...label}>
      <img className={art} src={src} alt="" />
      <img className={`${art} em-ghost em-cyan`} src={src} alt="" />
      <img className={`${art} em-ghost em-violet`} src={src} alt="" />
    </span>
  );
}

/** How long the big peek and a small one last (match the CSS animations). */
const BIG_MS = 4000;
const SMALL_MS = 2600;
/** Play time after which the plan starts again: only very long games see it all a second time. */
const CYCLE_MS = 10 * 60_000;

type Planned = { at: number; big: boolean };

/** One cycle: the big peek once (25 s–2.5 min in) and 3–5 small ones in the first 4 min, 15 s apart. */
function planCycle(start: number): Planned[] {
  const rand = (a: number, b: number) => a + Math.random() * (b - a);
  const plan: Planned[] = [{ at: start + rand(25_000, 150_000), big: true }];
  const small = 3 + Math.floor(Math.random() * 3);
  for (let tries = 0; plan.length < small + 1 && tries < 500; tries++) {
    const at = start + rand(8_000, 240_000);
    if (plan.every((p) => Math.abs(p.at - at) > 15_000)) plan.push({ at, big: false });
  }
  return plan.sort((a, b) => a.at - b.at);
}

/** True when the point shows the bare page background: no card, button, text or bordered box. */
function bare(x: number, y: number): boolean {
  for (let el = document.elementFromPoint(x, y); el; el = el.parentElement) {
    if (el.classList.contains('game-screen') || el === document.body) return true;
    if (el.matches('button, a, input, select, textarea, img, svg, canvas')) return false;
    if ([...el.childNodes].some((n) => n.nodeType === Node.TEXT_NODE && n.textContent?.trim())) return false;
    const cs = getComputedStyle(el);
    const clear = cs.backgroundColor === 'transparent' || cs.backgroundColor.endsWith(', 0)');
    if (!clear || cs.backgroundImage !== 'none' || (cs.borderStyle !== 'none' && cs.borderTopWidth !== '0px')) return false;
  }
  return true;
}

/** A spot below the game bar, in view, where a small face of w x h shows on bare background. */
function findSpot(w: number, h: number): { x: number; y: number } | null {
  const top = (document.querySelector('.game-nav')?.getBoundingClientRect().bottom ?? 0) + 8;
  const room = window.innerHeight - top - h;
  if (room < 0) return null;
  const probes: [number, number][] = [
    [0.5, 0.5],
    [0.15, 0.25],
    [0.85, 0.25],
    [0.15, 0.75],
    [0.85, 0.75],
  ];
  for (let i = 0; i < 60; i++) {
    const x = Math.random() * (window.innerWidth - w);
    const y = top + Math.random() * room;
    if (probes.every(([fx, fy]) => bare(x + fx * w, y + fy * h))) return { x, y };
  }
  return null;
}

/**
 * Only the Observer's face, without the hood, peeking at you during a game (docs/design.md,
 * Broadcast feel): its LED eyes and grin (assets/face.webp), behind the game. Once a game, at a
 * random moment, it comes in large in the top-right corner like a weak broadcast: looking away, it
 * turns to face you and keeps coming closer, the signal tears and blinks, and it fades. 3–5 times
 * more, small, somewhere on the bare background of the play area: it lifts its head, looks at you
 * and lowers it again. Only play time counts (`active`: not while the rules are open or the game is
 * over); after 10 minutes of play it all starts again. Nothing shows with reduced motion.
 */
export function Peeker({ active }: { active: boolean }) {
  const [peek, setPeek] = useState<{ big: boolean; x: number; y: number } | null>(null);
  const on = useRef(active);
  on.current = active;
  useEffect(() => {
    if (reducedMotion()) return;
    let played = 0;
    let cycle = 0;
    let plan = planCycle(0);
    let until = 0;
    let last = performance.now();
    const id = window.setInterval(() => {
      const now = performance.now();
      const dt = now - last;
      last = now;
      if (!on.current) {
        if (until) setPeek(null);
        until = 0;
        return;
      }
      played += dt;
      if (until) {
        if (played < until) return;
        setPeek(null);
        until = 0;
      }
      if (!plan.length && played >= cycle + CYCLE_MS) {
        cycle = played;
        plan = planCycle(played);
      }
      const next = plan[0];
      if (!next || played < next.at) return;
      plan.shift();
      if (next.big) {
        setPeek({ big: true, x: 0, y: 0 });
        until = played + BIG_MS;
        return;
      }
      const w = window.innerWidth >= 720 ? 112 : 84;
      const spot = findSpot(w, (w * 286) / 432);
      if (!spot) {
        plan.push({ at: played + 10_000, big: false });
        plan.sort((a, b) => a.at - b.at);
        return;
      }
      setPeek({ big: false, ...spot });
      until = played + SMALL_MS;
    }, 250);
    return () => window.clearInterval(id);
  }, []);
  const kind = peek && !peek.big ? 'small' : 'big';
  return (
    <span
      className={`peeker ${kind}${peek ? ' peek' : ''}`}
      style={peek && !peek.big ? { left: peek.x, top: peek.y } : undefined}
      hidden={!active}
      aria-hidden="true"
    >
      <img src={face} alt="" />
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
