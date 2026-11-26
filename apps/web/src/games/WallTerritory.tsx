// Wall Go: move a piece, build a wall, fence off the biggest area. Rules page: docs/games/wall-territory.md

import { useState } from 'react';
import { t } from '../i18n';
import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './WallTerritory.css';

type Dir = 'n' | 'e' | 's' | 'w';

interface WallView {
  size: number;
  me: number;
  seats: { id: string; area: number; sealed: boolean }[];
  pieces: { seat: number; index: number; x: number; y: number }[];
  east: boolean[];
  south: boolean[];
  owner: number[];
  turn: number;
  yourTurn: boolean;
  reach: number[][];
  last: [number, number, Dir] | null;
  turnNo: number;
  over: boolean;
}

const COLOURS = ['var(--seat-0)', 'var(--seat-1)', 'var(--seat-2)'];
const SIDES: { dir: Dir; label: string; key: string }[] = [
  { dir: 'n', label: 'Top', key: 'ArrowUp' },
  { dir: 'w', label: 'Left', key: 'ArrowLeft' },
  { dir: 'e', label: 'Right', key: 'ArrowRight' },
  { dir: 's', label: 'Bottom', key: 'ArrowDown' },
];

export default function WallTerritory({ view: raw }: ScreenProps) {
  const view = raw as WallView;
  const n = view.size;
  const [piece, setPiece] = useState<number | null>(null);
  const [dest, setDest] = useState<number | null>(null);
  const [forTurn, setForTurn] = useState(view.turnNo);
  if (forTurn !== view.turnNo) {
    setForTurn(view.turnNo);
    setPiece(null);
    setDest(null);
  }
  const mine = view.pieces.filter((p) => p.seat === view.me);
  const reach = new Set(piece === null ? [] : (view.reach[piece] ?? []));

  const canWall = (c: number, d: Dir) => {
    const x = c % n;
    const y = Math.floor(c / n);
    if (d === 'n') return y > 0 && !view.south[c - n];
    if (d === 's') return y < n - 1 && !view.south[c];
    if (d === 'w') return x > 0 && !view.east[c - 1];
    return x < n - 1 && !view.east[c];
  };

  const tap = (c: number) => {
    if (!view.yourTurn) return;
    const own = mine.find((p) => p.y * n + p.x === c);
    if (own && own.index !== piece) {
      setPiece(own.index);
      setDest(c);
    } else if (piece !== null && reach.has(c)) setDest(c);
  };
  const build = (d: Dir) => {
    if (piece === null || dest === null || !canWall(dest, d)) return;
    move({ type: 'turn', piece, x: dest % n, y: Math.floor(dest / n), wall: d });
  };

  useKeys((key) => {
    const side = SIDES.find((s) => s.key === key);
    if (side) build(side.dir);
    else if (key === 'Escape') {
      setPiece(null);
      setDest(null);
    } else return false;
    return true;
  });

  const moving = piece !== null ? mine.find((p) => p.index === piece) : undefined;
  const status = view.yourTurn
    ? piece === null
      ? t('Tap one of your pieces.')
      : t('Tap where it goes (or keep it), then pick a side for the wall.')
    : view.over
      ? t('Game over.')
      : t('{name} is building…', { name: nameOf(view.seats[view.turn]?.id ?? '') });

  return (
    <Play
      top={
        <GameTop
          title="Wall Go"
          step={`${t('Turn {n}', { n: view.turnNo })}${view.yourTurn ? ` · ${t('your move')}` : ''}`}
          score={view.seats[view.me]?.area ?? 0}
        />
      }
      stage={
        <svg className="drawing wall-territory-board" viewBox={`-0.1 -0.1 ${n + 0.2} ${n + 0.2}`} role="img" aria-label="Wall board">
          {view.owner
            .map((_, i) => i)
            .map((c) => {
              const x = c % n;
              const y = Math.floor(c / n);
              const owner = view.owner[c] ?? -1;
              return (
                // biome-ignore lint/a11y/useSemanticElements: an SVG square cannot be a <button>
                <rect
                  key={c}
                  x={x + 0.04}
                  y={y + 0.04}
                  width={0.92}
                  height={0.92}
                  rx={0.08}
                  className={`wall-territory-cell${reach.has(c) ? ' can' : ''}${dest === c ? ' dest' : ''}`}
                  style={owner >= 0 ? { fill: `color-mix(in srgb, ${COLOURS[owner]} 28%, var(--board))` } : undefined}
                  onClick={() => tap(c)}
                  role="button"
                  tabIndex={-1}
                  onKeyDown={() => {}}
                />
              );
            })}
          {view.east
            .flatMap((w, i) => (w ? [i] : []))
            .map((c) => (
              <line
                key={`e${c}`}
                x1={(c % n) + 1}
                y1={Math.floor(c / n)}
                x2={(c % n) + 1}
                y2={Math.floor(c / n) + 1}
                className="wall-territory-wall"
              />
            ))}
          {view.south
            .flatMap((w, i) => (w ? [i] : []))
            .map((c) => (
              <line
                key={`s${c}`}
                x1={c % n}
                y1={Math.floor(c / n) + 1}
                x2={(c % n) + 1}
                y2={Math.floor(c / n) + 1}
                className="wall-territory-wall"
              />
            ))}
          {view.pieces.map((p) => {
            const c = p === moving && dest !== null ? dest : p.y * n + p.x;
            const sel = p === moving;
            return (
              <circle
                key={`${p.seat}-${p.index}`}
                cx={(c % n) + 0.5}
                cy={Math.floor(c / n) + 0.5}
                r={0.3}
                style={{ fill: COLOURS[p.seat] }}
                className={`wall-territory-piece${p.seat === view.me ? ' mine' : ''}${sel ? ' sel' : ''}`}
              />
            );
          })}
          {view.last && (
            <rect
              x={(view.last[1] % n) + 0.04}
              y={Math.floor(view.last[1] / n) + 0.04}
              width={0.92}
              height={0.92}
              rx={0.08}
              className="wall-territory-last"
            />
          )}
        </svg>
      }
      controls={
        <>
          <ul className="list">
            {view.seats.map((s, i) => (
              <li key={s.id} className={i === view.turn && !view.over ? 'wall-territory-turn' : ''}>
                <span>
                  <i className="wall-territory-dot" style={{ background: COLOURS[i] }} /> {nameOf(s.id)}
                  {s.sealed && <span className="muted small"> · {t('sealed in')}</span>}
                </span>
                <span className="score">{s.area}</span>
              </li>
            ))}
          </ul>
          <p className="ask">{status}</p>
          <div className="label center">{t('Wall side')}</div>
          <div className="wall-territory-sides">
            {SIDES.map((s) => (
              <button
                key={s.dir}
                type="button"
                className={`btn wall-territory-${s.dir}`}
                disabled={dest === null || !canWall(dest, s.dir)}
                onClick={() => build(s.dir)}
              >
                {t(s.label)}
              </button>
            ))}
          </div>
          <KeyHint>{t('Arrow keys build the wall on that side; Esc cancels.')}</KeyHint>
        </>
      }
    />
  );
}
