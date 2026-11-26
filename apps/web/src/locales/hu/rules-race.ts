// Hungarian for Rules Race (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The parts of a secret rule (crates/rules/src/games/rules_race.rs, Cond and Effect).
const CONDITIONS: Record<string, string> = {
  'another player rolls a 1': 'egy másik játékos 1-est dob',
  'another player rolls a 3': 'egy másik játékos 3-ast dob',
  'another player rolls prison': 'egy másik játékos börtönt dob',
  'another player lands on a multiple of 4': 'egy másik játékos 4 többszörösére lép',
  'another player passes me': 'egy másik játékos megelőz',
};
const EFFECTS: Record<string, string> = {
  'I move forward 2': '2-t lépek előre',
  'they move back 2': 'ő 2-t lép vissza',
  'they go to prison': 'ő börtönbe kerül',
  'I swap places with them if they are ahead': 'helyet cserélek vele, ha előttem jár',
  'I get an escape card': 'kapok egy szabadulókártyát',
};
const RULES: Record<string, string> = {};
for (const [c, hc] of Object.entries(CONDITIONS)) {
  for (const [e, he] of Object.entries(EFFECTS)) RULES[`When ${c}, ${e}.`] = `Ha ${hc}, ${he}.`;
}
const esc = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const SEAT = '([a-z0-9-]+)';

// "Votes: you: keep, bot-1: switch 2, …": one part per racer, 4 on the page; every mix of "keep".
const VOTES: [string, string][] = Array.from({ length: 16 }, (_, mask) => {
  const en: string[] = [];
  const hu: string[] = [];
  let g = 0;
  for (let i = 0; i < 4; i++) {
    const keep = (mask >> i) & 1;
    en.push(keep ? `${SEAT}: keep` : `${SEAT}: switch (\\d)`);
    hu.push(keep ? `$${++g}: marad` : `$${++g}: váltás ($${++g}.)`);
  }
  return [`Votes: ${en.join(', ')}\\.`, `Szavazatok: ${hu.join(', ')}.`];
});

const hu: Locale = {
  name: 'Szabályverseny',
  short: 'Versenyezz a célig egy kockával, amin számok, börtön és szabadulás van; írj titkos szabályt, és szavazz a közös szabályokról.',
  strings: {
    ...CONDITIONS,
    ...EFFECTS,
    ...RULES,
    'Write your rule': 'Írd meg a szabályodat',
    'Group vote': 'Közös szavazás',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Race track': 'Versenypálya',
    GOAL: 'CÉL',
    Prison: 'Börtön',
    'escape card': 'szabadulókártya',
    'secret rule': 'titkos szabály',
    'writing…': 'ír…',
    'Waiting for the others to write their rules…': 'Várakozás, amíg a többiek megírják a szabályukat…',
    'Roll the die': 'Dobj a kockával',
    '{name} is rolling…': '{name} dob…',
    'Space or Enter rolls.': 'Szóközzel vagy Enterrel dobsz.',
    'Group rules': 'Közös szabályok',
    ON: 'BE',
    off: 'ki',
    'When…': 'Ha…',
    'then…': 'akkor…',
    'Write my secret rule': 'Megírom a titkos szabályom',
    'Vote sent. Waiting for the others…': 'Szavazat elküldve. Várakozás a többiekre…',
    'Switch one group rule?': 'Átkapcsolsz egy közös szabályt?',
    'Switch {n} off': '{n}. kikapcsolása',
    'Switch {n} on': '{n}. bekapcsolása',
    'Keep them all': 'Maradjon mind',
    // Group rules (GROUP_RULES).
    'Bump: landing on a square with other pieces sends them back 2.':
      'Lökés: ha olyan mezőre lépsz, ahol más bábuk állnak, azok 2-t visszalépnek.',
    'Strict prison: only an escape roll gets you out of prison.': 'Szigorú börtön: csak szabadulás dobással jöhetsz ki a börtönből.',
    'Exact finish: overshooting the goal bounces you back.': 'Pontos cél: ha túllépsz a célon, visszapattansz.',
    'Big three: a 3 moves 4.': 'Nagy hármas: a 3-as 4-et lép.',
    // Log lines without a seat.
    'Write your secret rule.': 'Írd meg a titkos szabályodat.',
    'Everyone has a secret rule. The race is on!': 'Mindenkinek van titkos szabálya. Indul a verseny!',
    'The group rules stay as they are.': 'A közös szabályok maradnak, ahogy vannak.',
  },
  // Log lines, the last roll and replay moments written by the rules. Seat ids stay (the screen
  // turns them into names afterwards), so the lines are worded to fit any name, "Te" too.
  patterns: [
    [`${SEAT} rolls (\\d)\\.`, '$1 – dobás: $2.'],
    [`${SEAT} rolls prison\\.`, '$1 – dobás: börtön.'],
    [`${SEAT} rolls escape\\.`, '$1 – dobás: szabadulás.'],
    [`${SEAT} rolled (\\d)`, '$1 – dobás: $2'],
    [`${SEAT} rolled prison`, '$1 – dobás: börtön'],
    [`${SEAT} rolled escape`, '$1 – dobás: szabadulás'],
    [`${SEAT} bumps ${SEAT} back to (\\d+)\\.`, 'Lökés ($1): $2 vissza, $3. mező.'],
    [`${SEAT} uses an escape card and stays out of prison\\.`, 'Szabadulókártya ($1): börtön kihagyva.'],
    [`${SEAT} goes to prison\\.`, 'Börtönbe: $1.'],
    [`${SEAT} escapes from prison\\.`, 'Kiszabadult a börtönből: $1.'],
    [`${SEAT} has served the time and is free\\.`, 'Letöltött büntetés, szabad: $1.'],
    [`${SEAT} gets an escape card\\.`, 'Szabadulókártya → $1.'],
    [`${SEAT}'s rule fires again\\.`, 'Szabály újra életbe lép ($1).'],
    ...Object.entries(RULES).map(([en, hu]): [string, string] => [`${SEAT}'s rule fires: ${esc(en)}`, `Szabály életbe lép ($1): ${hu}`]),
    ['Group rule (\\d) is now on\\.', '$1. közös szabály: bekapcsolva.'],
    ['Group rule (\\d) is now off\\.', '$1. közös szabály: kikapcsolva.'],
    ...VOTES,
    ['Votes: (.*)\\.', 'Szavazatok: $1.'],
    // Replay moments.
    ["Reaches the goal on (\\{[a-z0-9-]+\\})'s roll", 'Célba ér egy másik játékos dobására ($1)'],
    ['Rule pushes (\\{[a-z0-9-]+\\}) back off the goal', 'A szabály visszalöki a célból: $1'],
    ['Swap with (\\{[a-z0-9-]+\\}): from (\\d+) to (\\d+)', 'Helycsere ($1): $2. mezőről $3. mezőre'],
  ],
};

export default hu;
