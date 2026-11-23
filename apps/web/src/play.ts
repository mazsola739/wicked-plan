// The game being played, as a tiny store React reads with useSyncExternalStore.
// The rules run in the page through WebAssembly (crates/wasm): the player sits
// in seat "you", bots fill the other seats and move by themselves on tick.

import { useSyncExternalStore } from 'react';
import type { GameInfo, Moment, PlayerResult } from './types';
import init, { catalogue, LocalGame } from './wasm/wp_wasm.js';

const TICK_MS = 250;
/** How long a replay stays up unless it is skipped. */
const REPLAY_MS = 3200;

export interface PlayState {
  ready: boolean;
  games: GameInfo[];
  current: { gameId: string; view: unknown; results: PlayerResult[] | null } | null;
  /** A big moment being replayed: the game is frozen until it ends (docs/design.md, Replays). */
  replay: Moment | null;
  error: string | null;
}

let state: PlayState = { ready: false, games: [], current: null, replay: null, error: null };
let game: LocalGame | null = null;
let timer: ReturnType<typeof setInterval> | undefined;
let paused = false;
let lastSeq = 0;
let queue: Moment[] = [];
let replayTimer: ReturnType<typeof setTimeout> | undefined;
const listeners = new Set<() => void>();

function set(patch: Partial<PlayState>): void {
  state = { ...state, ...patch };
  for (const l of listeners) l();
}

function refresh(): void {
  if (!game || !state.current) return;
  const over = game.isOver();
  const results = over ? (JSON.parse(game.results()) as PlayerResult[]) : null;
  if (over) clearInterval(timer);
  set({ current: { ...state.current, view: JSON.parse(game.view()), results } });
  // New big moments are replayed once each; at most two wait, so a burst does not drag on.
  const fresh = (JSON.parse(game.moments()) as Moment[]).filter((m) => m.seq > lastSeq);
  if (fresh.length) {
    lastSeq = fresh[fresh.length - 1]?.seq ?? lastSeq;
    queue = [...queue, ...fresh].slice(-2);
    nextReplay();
  }
}

function nextReplay(): void {
  if (state.replay) return;
  const m = queue.shift();
  if (!m) return;
  set({ replay: m });
  replayTimer = setTimeout(endReplay, REPLAY_MS);
}

/** Ends the replay on screen (a tap or a key skips it); the game goes on, or the next one shows. */
export function endReplay(): void {
  clearTimeout(replayTimer);
  if (!state.replay) return;
  set({ replay: null });
  nextReplay();
}

export function start(gameId: string): void {
  stop();
  try {
    game = new LocalGame(gameId, Math.floor(Math.random() * 2 ** 32));
  } catch (e) {
    set({ error: e instanceof Error ? e.message : String(e) });
    return;
  }
  set({ current: { gameId, view: null, results: null }, error: null });
  refresh();
  let last = performance.now();
  timer = setInterval(() => {
    const now = performance.now();
    const ms = Math.round(now - last);
    last = now;
    // While paused (rules open) or replaying, no time passes, so the view cannot change.
    if (paused || state.replay) return;
    game?.tick(ms);
    refresh();
  }, TICK_MS);
}

export function stop(): void {
  clearInterval(timer);
  clearTimeout(replayTimer);
  game?.free();
  game = null;
  lastSeq = 0;
  queue = [];
  set({ current: null, replay: null });
}

const previews = new Map<string, unknown>();

/** The opening view of a game with a fixed seed, for the still preview on its picker card. */
export function previewView(gameId: string): unknown {
  if (!previews.has(gameId)) {
    const g = new LocalGame(gameId, 7);
    try {
      previews.set(gameId, JSON.parse(g.view()));
    } finally {
      g.free();
    }
  }
  return previews.get(gameId);
}

/** While paused (the rules are open) no game time passes, so timers and bots wait. */
export function setPaused(p: boolean): void {
  paused = p;
}

/** Sends one of your moves; a refused move shows its reason for a moment. */
export function move(m: unknown): void {
  if (!game || state.replay) return;
  try {
    game.apply(JSON.stringify(m));
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    set({ error: message });
    setTimeout(() => state.error === message && set({ error: null }), 2500);
  }
  refresh();
}

init().then(() => set({ ready: true, games: JSON.parse(catalogue()) as GameInfo[] }));

export function usePlay(): PlayState {
  return useSyncExternalStore(
    (l) => {
      listeners.add(l);
      return () => listeners.delete(l);
    },
    () => state,
  );
}
