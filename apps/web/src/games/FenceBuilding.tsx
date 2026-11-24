// Loop Fence Puzzle: one closed fence, numbers count the fenced sides; a race. Rules page: docs/games/fence-building.md

import { GameTop, KeyHint, move, Play, type ScreenProps, TimerBar } from './common';
import { RaceBar, type RaceView } from './PiecePuzzle';
import './FenceBuilding.css';

interface FenceView extends RaceView {
  size: number;
  clues: (number | null)[];
  /** 0 empty, 1 fence, 2 cross. */
  edges: number[];
}

/** Top, bottom, left, right edge of a plot (the same numbering as the rules). */
function sides(n: number, cell: number): [number, number, number, number] {
  const r = Math.floor(cell / n);
  const c = cell % n;
  const h = (n + 1) * n;
  return [r * n + c, (r + 1) * n + c, h + r * (n + 1) + c, h + r * (n + 1) + c + 1];
}

/** End points of an edge, in plot units. */
function ends(n: number, e: number): [number, number, number, number] {
  const h = (n + 1) * n;
  if (e < h) {
    const r = Math.floor(e / n);
    const c = e % n;
    return [c, r, c + 1, r];
  }
  const r = Math.floor((e - h) / (n + 1));
  const c = (e - h) % (n + 1);
  return [c, r, c, r + 1];
}

export default function FenceBuilding({ view: raw }: ScreenProps) {
  const view = raw as FenceView;
  const n = view.size;
  const cycle = (e: number) => move({ type: 'edge', edge: e, mark: ((view.edges[e] ?? 0) + 1) % 3 });
  const cells = Array.from({ length: n * n }, (_, i) => i);
  return (
    <Play
      top={
        <>
          <GameTop
            title="Loop Fence Puzzle"
            step={`Round ${view.round} of ${view.rounds} · ${n}×${n}`}
            score={view.racers[0]?.score ?? 0}
          />
          <TimerBar leftMs={view.timeLeftMs} totalMs={view.roundTimeMs} />
          <RaceBar view={view} />
        </>
      }
      stage={
        <svg className="drawing fb" viewBox={`-0.4 -0.4 ${n + 0.8} ${n + 0.8}`} role="img" aria-label="Field">
          {cells.map((cell) => {
            const k = view.clues[cell];
            if (k === null || k === undefined) return null;
            const fenced = sides(n, cell).filter((e) => view.edges[e] === 1).length;
            const state = fenced > k ? ' over' : fenced === k && k > 0 ? ' met' : '';
            return (
              <text key={`k${cell}`} x={(cell % n) + 0.5} y={Math.floor(cell / n) + 0.5} className={`fb-clue${state}`}>
                {k}
              </text>
            );
          })}
          {view.edges.map((m, e) => {
            if (m === 0) return null;
            const [x1, y1, x2, y2] = ends(n, e);
            if (m === 1) {
              // biome-ignore lint/suspicious/noArrayIndexKey: edges are positional
              return <line key={e} x1={x1} y1={y1} x2={x2} y2={y2} className="fb-fence" />;
            }
            const mx = (x1 + x2) / 2;
            const my = (y1 + y2) / 2;
            const d = 0.09;
            return (
              // biome-ignore lint/suspicious/noArrayIndexKey: edges are positional
              <path key={e} d={`M${mx - d} ${my - d}L${mx + d} ${my + d}M${mx + d} ${my - d}L${mx - d} ${my + d}`} className="fb-cross" />
            );
          })}
          {Array.from({ length: (n + 1) * (n + 1) }, (_, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: posts are positional
            <circle key={`p${i}`} cx={i % (n + 1)} cy={Math.floor(i / (n + 1))} r={0.07} className="fb-post" />
          ))}
          {/* Tap areas: each plot is split into four triangles, one per side. */}
          {cells.map((cell) => {
            const x = cell % n;
            const y = Math.floor(cell / n);
            const [top, bottom, left, right] = sides(n, cell);
            const c = `${x + 0.5},${y + 0.5}`;
            const tris: [number, string][] = [
              [top, `${x},${y} ${x + 1},${y} ${c}`],
              [bottom, `${x},${y + 1} ${x + 1},${y + 1} ${c}`],
              [left, `${x},${y} ${x},${y + 1} ${c}`],
              [right, `${x + 1},${y} ${x + 1},${y + 1} ${c}`],
            ];
            return tris.map(([e, pts]) => (
              // biome-ignore lint/a11y/useSemanticElements: an SVG triangle cannot be a <button>
              <polygon
                key={`t${cell}-${e}`}
                points={pts}
                className="fb-hit"
                role="button"
                tabIndex={-1}
                aria-label={`Edge ${e}`}
                onClick={() => cycle(e)}
                onKeyDown={(ev) => ev.key === 'Enter' && cycle(e)}
              />
            ));
          })}
        </svg>
      }
      controls={
        <>
          <p className="muted small center">
            Build one closed fence along the dotted lines. A number tells how many sides of its plot are fenced. Tap near a side: fence,
            then cross (a note), then empty.
          </p>
          <div className="row">
            <button type="button" className="btn ghost" onClick={() => move({ type: 'giveUp' })}>
              Give up this round
            </button>
            <button type="button" className="btn" onClick={() => move({ type: 'clear' })}>
              Clear all
            </button>
          </div>
          <KeyHint>Click near a side of a plot to fence it; click again for a cross.</KeyHint>
        </>
      }
    />
  );
}
