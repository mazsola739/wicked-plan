// Metal Cast Puzzle: slide every ring off the metal loop. Rules page: docs/games/metal-cast-puzzle.md

import { t } from '../i18n';
import { GameTop, KeyHint, move, Play, RoundDots, type ScreenProps, useKeys } from './common';
import './MetalCastPuzzle.css';

interface CastView {
  level: number;
  levels: number;
  /** true = on the loop; index 0 is ring 1 (the free end, on the right). */
  rings: boolean[];
  moves: number;
  best: number;
  solved: boolean[];
  score: number;
  over: boolean;
}

const GAP = 60;
const RAIL_Y = 70;

export default function MetalCastPuzzle({ view: raw }: ScreenProps) {
  const view = raw as CastView;
  const n = view.rings.length;
  const width = GAP * (n + 1) + 40;
  const toggle = (ring: number) => move({ type: 'toggle', ring });
  // Desktop: digits slide a ring, Backspace resets.
  useKeys((key) => {
    const ring = Number(key);
    if (/^[1-9]$/.test(key) && ring <= n) toggle(ring);
    else if (key === 'Backspace') move({ type: 'reset' });
    else return false;
    return true;
  });
  // Ring 1 is drawn on the right, next to the free end of the loop.
  const xOf = (i: number) => 40 + GAP * (n - i);
  const onCount = view.rings.filter(Boolean).length;
  return (
    <Play
      top={
        <>
          <GameTop
            title="Metal Cast Puzzle"
            step={t('Level {n} of {total} · {rings} rings', { n: view.level, total: view.levels, rings: n })}
            score={view.score}
          />
          <RoundDots results={view.solved} total={view.levels} />
        </>
      }
      stage={
        <svg className="drawing mcp" viewBox={`0 0 ${width} 210`} role="img" aria-label={t('Rings on a metal loop')}>
          <defs>
            <linearGradient id="mcp-steel" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" style={{ stopColor: 'var(--silver)' }} />
              <stop offset="1" style={{ stopColor: 'color-mix(in srgb, var(--silver) 50%, var(--board))' }} />
            </linearGradient>
          </defs>
          {/* The bar that holds the posts. */}
          <rect x={20} y={182} width={width - 40} height={10} rx={5} className="mcp-bar" />
          {view.rings.map((on, i) => {
            const x = xOf(i);
            const ringY = on ? RAIL_Y : 132;
            return (
              // biome-ignore lint/a11y/useSemanticElements: an SVG ring cannot be a <button>
              <g
                // biome-ignore lint/suspicious/noArrayIndexKey: rings are positional
                key={i}
                className={`mcp-ring${on ? ' on' : ''}`}
                role="button"
                tabIndex={0}
                aria-label={on ? t('Ring {n} on', { n: i + 1 }) : t('Ring {n} off', { n: i + 1 })}
                onClick={() => toggle(i + 1)}
                onKeyDown={(e) => e.key === 'Enter' && toggle(i + 1)}
              >
                <rect x={x - GAP / 2} y={20} width={GAP} height={180} className="mcp-hit" />
                <line x1={x} y1={ringY + 22} x2={x} y2={186} className="mcp-post" />
                {/* Back half of the ring, behind the loop. */}
                <path d={`M ${x - 20} ${ringY} A 20 24 0 0 1 ${x + 20} ${ringY}`} className="mcp-wire back" />
                <text x={x} y={204} className="mcp-num">
                  {i + 1}
                </text>
              </g>
            );
          })}
          {/* The long loop: handle on the left, free end on the right. */}
          <path d={`M ${width - 22} ${RAIL_Y - 10} H 34 a 10 10 0 0 0 0 20 H ${width - 22}`} className="mcp-loop" fill="none" />
          <rect x={6} y={RAIL_Y - 18} width={26} height={36} rx={8} className="mcp-handle" />
          {view.rings.map((on, i) => {
            const x = xOf(i);
            const ringY = on ? RAIL_Y : 132;
            return (
              <path
                // biome-ignore lint/suspicious/noArrayIndexKey: rings are positional
                key={i}
                d={`M ${x - 20} ${ringY} A 20 24 0 0 0 ${x + 20} ${ringY}`}
                className={`mcp-wire front${on ? ' on' : ''}`}
                pointerEvents="none"
              />
            );
          })}
        </svg>
      }
      controls={
        <>
          <p className="muted small center">
            {t('Ring 1 always slides. Any other ring slides only when the ring just below it is')} <strong>{t('on')}</strong>{' '}
            {t('and every ring below that one is')} <strong>{t('off')}</strong>. {t('Take every ring off.')}
          </p>
          <div className="mcp-keys" style={{ gridTemplateColumns: `repeat(${n}, 1fr)` }}>
            {view.rings
              .map((on, i) => ({ on, ring: i + 1 }))
              .reverse()
              .map(({ on, ring }) => (
                <button key={ring} type="button" className={`key${on ? ' ok' : ''}`} onClick={() => toggle(ring)}>
                  {ring}
                </button>
              ))}
          </div>
          <p className="center">
            {t(onCount === 1 ? '{n} ring on' : '{n} rings on', { n: onCount })} ·{' '}
            {t(view.moves === 1 ? '{n} move' : '{n} moves', { n: view.moves })} · {t('best possible {n}', { n: view.best })}
          </p>
          <div className="row">
            <button type="button" className="btn ghost" onClick={() => move({ type: 'skip' })}>
              {t('Give up')}
            </button>
            <button type="button" className="btn" onClick={() => move({ type: 'reset' })}>
              {t('Reset level')}
            </button>
          </div>
          <KeyHint>{t('Keys: 1–{n} slide that ring, Backspace resets.', { n })}</KeyHint>
        </>
      }
    />
  );
}
