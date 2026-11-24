// Shapes of what the rules (crates/rules, through crates/wasm) send to the page.

export interface GameInfo {
  id: string;
  kind: string;
  difficulty: 'easy' | 'normal' | 'medium' | 'hard' | 'expert';
  /** 1-3 categories (docs/games.md, Categories). */
  tags: string[];
  minPlayers: number;
  maxPlayers: number;
  name: string;
  short: string;
}

/** A big moment worth a replay (crates/rules game.rs `Moment`); `{seat-id}` in the text is a name. */
export interface Moment {
  seq: number;
  by: string;
  text: string;
}

export interface PlayerResult {
  playerId: string;
  rank: number;
  points: number;
}

// Quiz games (cubes, rectangles, equation target).
export interface QuizHistory<P, A> {
  puzzle: P;
  given: A | null;
  solution: A;
  correct: boolean;
  points: number;
}

export interface QuizView<P, A> {
  round: number;
  rounds: number;
  puzzle: P | null;
  timeLeftMs: number;
  roundTimeMs: number;
  history: QuizHistory<P, A>[];
  score: number;
  over: boolean;
}

export interface CubesPuzzle {
  cols: number;
  rows: number;
  heights: number[];
}

export interface RectanglesPuzzle {
  cols: number;
  rows: number;
  h: boolean[][];
  v: boolean[][];
}

export type Op = '+' | '-' | '*';
export interface EquationPuzzle {
  cards: number[];
  target: number;
}
export interface EquationAnswer {
  picks: [number, number, number];
  ops: [Op, Op];
}

export interface HexView {
  numbers: number[] | null;
  studying: boolean;
  round: number;
  rounds: number;
  target: number | null;
  timeLeftMs: number;
  phaseTimeMs: number;
  history: { target: number; given: number[] | null; solution: number[]; correct: boolean; points: number }[];
  score: number;
  over: boolean;
}

export interface KnightView {
  board: number;
  boards: number;
  size: number;
  path: [number, number][];
  targets: [number, number][];
  stuck: boolean;
  restarts: number;
  solved: number;
  score: number;
  timeLeftMs: number;
  totalTimeMs: number;
  over: boolean;
}

export interface MagicView {
  board: number;
  boards: number;
  size: number;
  magicSum: number;
  cells: number[];
  given: boolean[];
  wrongTries: number;
  solved: boolean[];
  score: number;
  timeLeftMs: number;
  boardTimeMs: number;
  over: boolean;
}
