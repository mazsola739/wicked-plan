// Crooked Cops: a chase on a metro map with a secret corrupt officer. Rules: docs/games/thieves-and-cops.md

import { GameTop, KeyHint, move, nameOf, Play, type ScreenProps, useKeys } from './common';
import './ThievesAndCops.css';

type Role = 'thief' | 'cop' | 'corrupt';

interface AgentView {
  index: number;
  id: string;
  role: 'thief' | 'police' | 'corrupt';
  pos: number | null;
  caught: boolean;
  kept: number;
  catches: number;
  seen: [number, number] | null;
  done: boolean;
}

interface TcView {
  phase: 'move' | 'unmask' | 'reveal' | 'over';
  round: number;
  rounds: number;
  me: number;
  role: Role;
  stations: { name: string; x: number; y: number }[];
  edges: [number, number, number][];
  agents: AgentView[];
  loot: number[] | null;
  taken: [number, number][];
  keptTotal: number;
  toWin: number;
  options: number[];
  chosen: number | null;
  canAccuse: boolean;
  accused: number | null;
  unmasked: boolean;
  winner: 'thieves' | 'police' | null;
  log: string[];
  score: number;
}

// Ring, red, blue, green and purple lines (crates/rules thieves_and_cops.rs).
const LINE_COLOURS = ['var(--g-yellow)', 'var(--g-red)', 'var(--g-blue)', 'var(--g-green)', 'var(--g-purple)'];
const ROLE_TEXT: Record<Role, string> = {
  thief: 'Thief: collect pieces with the other thieves and stay away from the police.',
  cop: 'Police: catch the thieves. One of your colleagues is corrupt.',
  corrupt: 'Corrupt officer: act like police, but help the thieves win. You never catch anyone.',
};

// Faceted pieces (docs/design.md, "Icons are faceted"), lit from the top left.
const pt = (a: number, r: number) => `${(r * Math.cos(a)).toFixed(1)},${(r * Math.sin(a)).toFixed(1)}`;
const OCT = (r: number) => Array.from({ length: 8 }, (_, i) => pt(Math.PI / 8 + (Math.PI / 4) * i, r));
// An agent token is a cut octagon: each bevel facet white toward the light, black away from it.
const OUTER = OCT(13);
const INNER = OCT(8.5);
const TOKEN_EDGE = `M${OUTER.join(' ')}Z`;
const TOKEN_FACETS = OUTER.flatMap((p, i) => {
  const j = (i + 1) % 8;
  const d = Math.cos((Math.PI / 4) * j + (3 * Math.PI) / 4); // this facet faces 45° × j
  if (Math.abs(d) < 0.2) return [];
  const light = d > 0 ? { fill: '#fff', fillOpacity: 0.1 + 0.35 * d } : { fill: '#000', fillOpacity: 0.12 + 0.23 * -d };
  return [{ d: `M${p} ${OUTER[j]} ${INNER[j]} ${INNER[i]}Z`, ...light }];
});

/** The loot: a gold jewel cut like a gem, centred on a station. */
function Loot({ x, y }: { x: number; y: number }) {
  return (
    <g className="tc-loot" transform={`translate(${x} ${y})`}>
      <path className="edge" d="M-5,-8 5,-8 10,-3 0,10 -10,-3Z" />
      <path fill="#fff" fillOpacity=".5" d="M-5,-8 5,-8 3,-3 -3,-3Z" />
      <path fill="#fff" fillOpacity=".3" d="M-10,-3 -5,-8 -3,-3Z" />
      <path fill="#fff" fillOpacity=".15" d="M5,-8 10,-3 3,-3Z M-10,-3 -3,-3 0,10Z" />
      <path fill="#000" fillOpacity=".12" d="M-3,-3 3,-3 0,10Z" />
      <path fill="#000" fillOpacity=".32" d="M3,-3 10,-3 0,10Z" />
      <path fill="#fff" d="M-3,-7 -2,-6 -3,-5 -4,-6Z" />
    </g>
  );
}

function names(line: string): string {
  return line.replace(/\b(you|bot-\d+)\b/g, (id) => nameOf(id));
}

export default function ThievesAndCops({ view }: ScreenProps) {
  const v = view as TcView;
  const me = v.agents[v.me];
  const end = v.phase === 'reveal' || v.phase === 'over';
  const go = (to: number) => v.options.includes(to) && move({ type: 'go', to });
  useKeys((key) => {
    if ((key === 's' || key === 'S') && me?.pos != null) go(me.pos);
    else if (key === 'Enter' && end) move({ type: 'close' });
    else return false;
    return true;
  });
  const tokens = new Map<number, AgentView[]>();
  for (const a of v.agents) {
    if (a.pos === null || (a.caught && !end)) continue;
    tokens.set(a.pos, [...(tokens.get(a.pos) ?? []), a]);
  }
  const ghosts = v.agents.filter((a) => a.role === 'thief' && a.pos === null && a.seen && !a.caught);
  const officers = v.agents.filter((a) => a.role !== 'thief' && a.index !== v.me);
  return (
    <Play
      top={
        <GameTop
          title="Crooked Cops"
          step={end ? 'Everything revealed' : v.phase === 'unmask' ? 'Unmask the corrupt officer' : `Round ${v.round} of ${v.rounds}`}
          score={v.score}
        />
      }
      stage={
        <svg className="drawing tc-map" viewBox="20 30 560 550" role="img" aria-label="Metro map">
          {v.edges.map(([a, b, line]) => {
            const p = v.stations[a];
            const q = v.stations[b];
            return p && q ? (
              <line key={`${a}-${b}`} x1={p.x} y1={p.y} x2={q.x} y2={q.y} style={{ stroke: LINE_COLOURS[line] }} className="tc-line" />
            ) : null;
          })}
          {v.stations.map((s, i) => {
            const can = v.options.includes(i);
            const taken = v.taken.some(([, st]) => st === i);
            return (
              // biome-ignore lint/a11y/useSemanticElements: an SVG station cannot be a <button>
              <g
                key={s.name}
                className={`tc-station${can ? ' can' : ''}${v.chosen === i ? ' chosen' : ''}`}
                onClick={() => go(i)}
                role="button"
                tabIndex={can ? 0 : -1}
                onKeyDown={(e) => e.key === 'Enter' && go(i)}
              >
                <circle cx={s.x} cy={s.y} r={can ? 24 : 15} className="tc-hit" />
                <circle cx={s.x} cy={s.y} r={13} className="tc-dot" />
                {v.loot?.includes(i) && <Loot x={s.x} y={s.y} />}
                {taken && (
                  <text x={s.x} y={s.y + 5} className="tc-taken">
                    ✕
                  </text>
                )}
                <text x={s.x} y={s.y + 32} className="tc-name">
                  {s.name}
                </text>
              </g>
            );
          })}
          {ghosts.map((a) => {
            const [round, st] = a.seen ?? [0, 0];
            const s = v.stations[st];
            return s ? (
              <g key={`ghost-${a.id}`} className="tc-ghost">
                <circle cx={s.x + 16} cy={s.y - 18} r={11} />
                <text x={s.x + 16} y={s.y - 14}>
                  R{round}
                </text>
              </g>
            ) : null;
          })}
          {[...tokens.entries()].flatMap(([st, list]) => {
            const s = v.stations[st];
            if (!s) return [];
            return list.map((a, k) => {
              const x = s.x + k * 22 - (list.length - 1) * 11;
              const y = s.y - 28;
              const cls = a.role === 'thief' ? 'thief' : a.role === 'corrupt' ? 'corrupt' : 'police';
              return (
                <g
                  key={a.id}
                  className={`tc-token ${cls}${a.index === v.me ? ' me' : ''}${a.caught ? ' caught' : ''}`}
                  transform={`translate(${x} ${y})`}
                >
                  <path className="edge" d={TOKEN_EDGE} />
                  {TOKEN_FACETS.map((f) => (
                    <path key={f.d} {...f} />
                  ))}
                  <path fill="#fff" d="M-7,-8 -6,-7 -7,-6 -8,-7Z" />
                  <text y={4}>{nameOf(a.id)[0]}</text>
                </g>
              );
            });
          })}
        </svg>
      }
      controls={
        <>
          <section className={`tc-card tc-${v.role}`}>
            <strong>
              {nameOf(me?.id ?? 'you')} · {v.role === 'cop' ? 'police' : v.role}
            </strong>
            <p className="small">{ROLE_TEXT[v.role]}</p>
            <p className="small muted">
              Pieces kept by the thieves: <strong>{v.keptTotal}</strong> of {v.toWin} needed
            </p>
          </section>
          {v.phase === 'move' &&
            (me?.caught ? (
              <p className="muted center">You were caught.</p>
            ) : v.options.length > 0 ? (
              <>
                <p className="muted small center">Tap a lit station to move one stop, or stay.</p>
                <button type="button" className="btn primary" onClick={() => me?.pos != null && go(me.pos)}>
                  Stay at {me?.pos != null ? v.stations[me.pos]?.name : ''}
                </button>
                <KeyHint>S stays.</KeyHint>
              </>
            ) : (
              <p className="muted center">Waiting for the others…</p>
            ))}
          {v.phase === 'unmask' &&
            (v.canAccuse ? (
              <div className="tc-accuse">
                <p className="muted small center">Who is the corrupt officer?</p>
                {officers.map((a) => (
                  <button key={a.id} type="button" className="btn" onClick={() => move({ type: 'accuse', target: a.index })}>
                    {nameOf(a.id)}
                  </button>
                ))}
              </div>
            ) : (
              <p className="muted center">{v.role === 'thief' ? 'The police are voting…' : 'Vote sent. Waiting…'}</p>
            ))}
          {end && (
            <>
              <p className={`last ${(v.winner === 'thieves') === (v.role !== 'cop') ? 'ok' : 'bad'}`}>
                {v.winner === 'thieves' ? 'The thieves win.' : 'The police win.'}
                {v.unmasked ? ' The corrupt officer was unmasked.' : ''} Your score: {v.score}
              </p>
              <button type="button" className="btn primary" onClick={() => move({ type: 'close' })}>
                See results
              </button>
            </>
          )}
          <ul className="list tc-players">
            {v.agents.map((a) => (
              <li key={a.id} className={a.index === v.me ? 'me' : ''}>
                <span>
                  <span className={`tc-badge ${a.role}`}>{a.role === 'thief' ? 'T' : a.role === 'corrupt' ? 'C' : 'P'}</span> {nameOf(a.id)}
                  {a.caught ? ' · caught' : ''}
                </span>
                <small className="muted">
                  {a.role === 'thief' ? `${a.kept} piece${a.kept === 1 ? '' : 's'}` : `${a.catches} catch${a.catches === 1 ? '' : 'es'}`}
                  {!end && !a.done ? ' · …' : ''}
                </small>
              </li>
            ))}
          </ul>
          <div className="tc-log">
            {v.log
              .slice(-5)
              .reverse()
              .map((line, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: the log only grows at the end
                <p key={i} className="small">
                  {names(line)}
                </p>
              ))}
          </div>
        </>
      }
    />
  );
}
