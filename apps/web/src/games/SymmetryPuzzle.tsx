// Symmetrical Puzzle: paint the missing mirror cells, a race. Rules page: docs/games/symmetry-puzzle.md

import { useState } from 'react';
import { t } from '../i18n';
import { GameTop, KeyHint, move, Play, type ScreenProps, TimerBar, useKeys } from './common';
import { RaceBar, type RaceView } from './PiecePuzzle';
import './SymmetryPuzzle.css';

type Axis = 'vertical' | 'diagonal' | 'both';

interface SymmetryView extends RaceView {
  size: number;
  axis: Axis;
  colours: number;
  cells: number[];
  given: boolean[];
}

const PAINT = ['transparent', 'var(--g-yellow)', 'var(--g-blue)', 'var(--g-red)'];
const AXIS_TEXT: Record<Axis, string> = {
  vertical: 'the vertical axis',
  diagonal: 'the diagonal axis',
  both: 'both axes',
};

export default function SymmetryPuzzle({ view: raw }: ScreenProps) {
  const view = raw as SymmetryView;
  const [colour, setColour] = useState(1);
  const n = view.size;
  const paint = (cell: number) => {
    if (view.given[cell]) return;
    move({ type: 'paint', cell, colour: view.cells[cell] === colour ? 0 : colour });
  };
  useKeys((key) => {
    const c = Number(key);
    if (/^[0-9]$/.test(key) && c <= view.colours) setColour(c);
    else return false;
    return true;
  });
  const lines: [number, number, number, number][] = [];
  if (view.axis === 'vertical' || view.axis === 'both') lines.push([n / 2, 0, n / 2, n]);
  if (view.axis === 'both') lines.push([0, n / 2, n, n / 2]);
  if (view.axis === 'diagonal') lines.push([0, 0, n, n]);
  return (
    <Play
      top={
        <>
          <GameTop
            title="Symmetrical Puzzle"
            step={t('Round {n} of {total}', { n: view.round, total: view.rounds })}
            score={view.racers[0]?.score ?? 0}
          />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.roundTimeMs} />
          <RaceBar view={view} />
        </>
      }
      stage={
        <div className="sym-wrap">
          <div className="board sym" style={{ gridTemplateColumns: `repeat(${n}, 1fr)` }}>
            {view.cells.map((c, i) => (
              <button
                // biome-ignore lint/suspicious/noArrayIndexKey: cells are positional
                key={i}
                type="button"
                className={`sq sym-cell${view.given[i] ? ' given' : ''}`}
                style={c ? { background: PAINT[c] } : undefined}
                disabled={view.given[i]}
                onClick={() => paint(i)}
                aria-label={t('Cell {n}', { n: i + 1 })}
              />
            ))}
          </div>
          <svg className="sym-axis" viewBox={`0 0 ${n} ${n}`} preserveAspectRatio="none" aria-hidden="true">
            {lines.map(([x1, y1, x2, y2]) => (
              <line key={`${x1},${y1}`} x1={x1} y1={y1} x2={x2} y2={y2} />
            ))}
          </svg>
        </div>
      }
      controls={
        <>
          <p className="muted small center">
            {t('Make the figure symmetric about {axis}. The cells with a dot are given; add only what the mirror needs.', {
              axis: t(AXIS_TEXT[view.axis]),
            })}
          </p>
          <div className="sym-palette" style={{ gridTemplateColumns: `repeat(${view.colours + 1}, 1fr)` }}>
            {Array.from({ length: view.colours }, (_, k) => k + 1).map((c) => (
              <button
                key={c}
                type="button"
                className={`key sym-colour${colour === c ? ' sel' : ''}`}
                style={{ background: PAINT[c] }}
                onClick={() => setColour(c)}
                aria-label={t('Colour {n}', { n: c })}
              >
                {c}
              </button>
            ))}
            <button type="button" className={`key sym-colour erase${colour === 0 ? ' sel' : ''}`} onClick={() => setColour(0)}>
              ⌫
            </button>
          </div>
          <button type="button" className="btn ghost" onClick={() => move({ type: 'giveUp' })}>
            {t('Give up this round')}
          </button>
          <KeyHint>{t('Keys: 1–{n} pick a colour, 0 the eraser. Tap a cell to paint it.', { n: view.colours })}</KeyHint>
        </>
      }
    />
  );
}
