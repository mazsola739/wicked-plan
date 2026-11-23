// The game being played, as a tiny store React reads with useSyncExternalStore.
// The rules run in the page through WebAssembly (crates/wasm): the player sits
// in seat "you", bots fill the other seats and move by themselves on tick.

import { useSyncExternalStore } from 'react';
import type { GameInfo, PlayerResult } from './types';
import init, { catalogue, LocalGame } from './wasm/wp_wasm.js';

const TICK_MS = 250;

export interface PlayState {
  ready: boolean;
  games: GameInfo[];
  current: { gameId: string; view: unknown; results: PlayerResult[] | null } | null;
  error: string | null;
}

let state: PlayState = { ready: false, games: [], current: null, error: null };
let game: LocalGame | null = null;
let timer: ReturnType<typeof setInterval> | undefined;
let paused = false;
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
    if (!paused) game?.tick(Math.round(now - last));
    last = now;
    refresh();
  }, TICK_MS);
}

export function stop(): void {
  clearInterval(timer);
  game?.free();
  game = null;
  set({ current: null });
}

/** While paused (the rules are open) no game time passes, so timers and bots wait. */
export function setPaused(p: boolean): void {
  paused = p;
}

/** Sends one of your moves; a refused move shows its reason for a moment. */
export function move(m: unknown): void {
  if (!game) return;
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
