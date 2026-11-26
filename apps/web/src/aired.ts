// Where each game was played on the show: season and day, the final, an exile match or a hidden
// stage, as the English Wikipedia article on The Devil's Plan lists them (checked 2026-11-26; it
// gives days, not episode numbers). Shown small in the corner of each game's card. Games the
// article does not list have no mark.

const AIRED: Record<string, string> = {
  // Season 1 (2023)
  'virus-game': 'S01 · D01',
  'spinning-puzzle-wheel': 'S01 · D01',
  'rules-race': 'S01 · D02',
  'memory-scene-quiz': 'S01 · D02',
  'secret-number': 'S01 · D03',
  'word-block-tower': 'S01 · D03',
  'animal-tile-auction': 'S01 · D04',
  'weight-balance-puzzle': 'S01 · D04',
  'polyomino-territory': 'S01 · D05',
  'duplicate-photo-buzzer': 'S01 · D05',
  'equation-hi-lo': 'S01 · D06',
  'hidden-fourth-player': 'S01 · D06',
  'nine-mens-morris': 'S01 · FINAL',
  hexagon: 'S01 · FINAL',
  'blind-gomoku': 'S01 · HIDDEN',
  // Season 2: Death Room (2025)
  'thieves-and-cops': 'S02 · D01',
  'minus-one': 'S02 · D01',
  'hidden-rules-race': 'S02 · D02',
  'time-auction': 'S02 · D02',
  'monster-alliance': 'S02 · D03',
  'sniper-holdem': 'S02 · D03',
  'treasure-island': 'S02 · D04',
  'dice-prediction': 'S02 · D04',
  'colour-mancala': 'S02 · D05',
  'wall-territory': 'S02 · D05',
  'colour-count-bluff': 'S02 · D06',
  'equation-target': 'S02 · D06',
  'big-or-small': 'S02 · FINAL',
  'tiger-and-goat': 'S02 · FINAL',
  'question-and-truth': 'S02 · FINAL',
  'knights-tour': 'S02 · HIDDEN',
  // Season 3: Dead Again (2026)
  'either-or': 'S03 · D01',
  'count-the-cubes': 'S03 · D01',
  'count-the-rectangles': 'S03 · D01',
  'lay-the-grass': 'S03 · D01',
  'memory-poker': 'S03 · D01',
  'heavy-13': 'S03 · D01',
  'piece-puzzle': 'S03 · EXILE 1',
  'mystery-stock-market': 'S03 · D02',
  'find-the-differences': 'S03 · EXILE 2',
  'russian-roulette': 'S03 · D02',
  'secret-code': 'S03 · EXILE 3',
  'jewel-black-market': 'S03 · D03',
  'fence-building': 'S03 · EXILE 4',
  'piggy-bank': 'S03 · D03',
  'auction-bingo': 'S03 · D04',
  'symmetry-puzzle': 'S03 · EXILE 5',
  'magic-square': 'S03 · HIDDEN',
};

export const airedOf = (id: string): string | undefined => AIRED[id];

/** The season as "1", "2" or "3"; "" when the game has no mark. */
export const seasonOf = (id: string): string => AIRED[id]?.slice(2, 3) ?? '';

/** Where in the season: "d1" … "d6", "final", "exile" or "hidden"; "" when the game has no mark. */
export function stageOf(id: string): string {
  const stage = AIRED[id]?.split(' · ')[1];
  if (!stage) return '';
  return stage.startsWith('D') ? `d${Number(stage.slice(1))}` : (stage.split(' ')[0] ?? '').toLowerCase();
}
