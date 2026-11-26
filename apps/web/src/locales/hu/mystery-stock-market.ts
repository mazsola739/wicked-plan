// Hungarian for Mystery Stock Market (i18n.ts). The English text is the key. The made-up company
// names and tickers stay as they are.

import type { Locale } from '../../i18n';

// The ten rules (crates/rules/src/games/mystery_stock_market.rs, RULE_TEXTS): name and text.
const RULES: [string, string, string, string][] = [
  ['Steady', 'Egyenletes', '+5% every round', '+5% minden körben'],
  ['Seesaw', 'Libikóka', '+20% after odd rounds, −15% after even rounds', '+20% a páratlan körök után, −15% a páros körök után'],
  [
    'Crowd',
    'Tömeg',
    '+4% per share bought by everyone this round, −4% per share sold (max ±40%)',
    '+4% minden részvényért, amit ebben a körben bárki vett, −4% minden eladottért (legfeljebb ±40%)',
  ],
  ['Contrarian', 'Szembemenő', 'the opposite of Crowd', 'a Tömeg ellentéte'],
  ['Shadow', 'Árnyék', 'the same change as one other stock this round', 'ugyanannyit változik, mint egy másik részvény ebben a körben'],
  [
    'Mirror',
    'Tükör',
    'the opposite change of one other stock this round',
    'az ellenkezőjére változik, mint egy másik részvény ebben a körben',
  ],
  [
    'Bubble',
    'Buborék',
    '+12% a round until a secret round (12–18), then −60% once, then −5% a round',
    'körönként +12% egy titkos körig (12–18), aztán egyszer −60%, utána körönként −5%',
  ],
  ['Dice', 'Kocka', 'a random change from −25% to +25%', 'véletlen változás −25% és +25% között'],
  [
    'Lonely',
    'Magányos',
    '−10% when more than half the players hold it, +12% otherwise',
    '−10%, ha a játékosok több mint fele tartja, különben +12%',
  ],
  ['Calendar', 'Naptár', '+30% after every 4th round, −6% after the others', '+30% minden 4. kör után, −6% a többi után'],
];

const escapeRe = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

// The exact rule of one stock (Rule::text): fixed for most rules, with a stock or a round for three.
const EXACT: [string, string, string][] = [
  ...RULES.filter(([en]) => !['Shadow', 'Mirror', 'Bubble'].includes(en)).map(([en, hu, text, huText]): [string, string, string] => [
    escapeRe(`${en}: ${text}`),
    `${hu}: ${huText}`,
    `${hu} (${huText})`,
  ]),
  ['Shadow: moves like (.+)', 'Árnyék: úgy mozog, mint {g}', 'Árnyék (úgy mozog, mint {g})'],
  ['Mirror: moves opposite to (.+)', 'Tükör: ellentétesen mozog, mint {g}', 'Tükör (ellentétesen mozog, mint {g})'],
  [
    'Bubble: \\+12% a round, crashes −60% after round (\\d+), then −5% a round',
    'Buborék: körönként +12%, a {g}. kör után −60%-ot zuhan, utána körönként −5%',
    'Buborék (körönként +12%, a {g}. kör után −60%-ot zuhan, utána körönként −5%)',
  ],
];

// The public clues in the log come first: "(.+) follows" would match them too.
const rulePatterns: [string, string][] = [
  ...EXACT.map(([en, , inClue]): [string, string] => [
    `Clue for everyone: (.+) follows ${en}\\.`,
    `Nyom mindenkinek: $1 szabálya: ${inClue.replace('{g}', '$2')}.`,
  ]),
  // Your clue: "<stock> follows <rule>".
  ...EXACT.map(([en, , inClue]): [string, string] => [`(.+) follows ${en}`, `$1 szabálya: ${inClue.replace('{g}', '$2')}`]),
  ...EXACT.map(([en, hu]): [string, string] => [en, hu.replace('{g}', '$1')]),
];

const hu: Locale = {
  name: 'Rejtélyes tőzsde',
  short: 'Kereskedj hét kitalált részvénnyel 24 körön át, és fejtsd meg a titkos szabályokat, amelyek az árfolyamukat mozgatják.',
  strings: {
    ...Object.fromEntries(
      RULES.flatMap(([en, hu, text, huText]) => [
        [en, hu],
        [text, huText],
      ]),
    ),
    // The screen.
    'Price chart': 'Árfolyamgrafikon',
    'Market closed': 'A tőzsde zárt',
    'Round {n} of {total}': '{n}. kör / {total}',
    Cash: 'Készpénz',
    Worth: 'Vagyon',
    'you hold {n}': 'nálad: {n}',
    'Last round:': 'Előző kör:',
    '{n} shares net': 'nettó {n} részvény',
    'held by {n} of {total}': '{total} játékosból {n} tartja',
    'Sell {n}': 'Eladás: {n}',
    'Buy {n}': 'Vétel: {n}',
    'Sell all': 'Mindet eladom',
    'Buy max ({n})': 'Maximum vétel ({n})',
    'Done for this round': 'Kész ebben a körben',
    'See results': 'Eredmények',
    'Keys: 1–7 pick a stock, B buys one, S sells one, Enter is Done.':
      'Billentyűk: 1–7 részvényt választ, B vesz egyet, S elad egyet, Enter: kész.',
    'Your clue: {clue}.': 'A nyomod: {clue}.',
    'The ten possible rules': 'A tíz lehetséges szabály',

    // The log (the rules).
    'The market opens. Every stock costs 10,000.': 'Nyit a tőzsde. Minden részvény 10,000-be kerül.',
    'The market is closed. All rules revealed.': 'A tőzsde zárt. Minden szabály kiderült.',
  },
  patterns: [
    ...rulePatterns,
    ['Round (\\d+): best (\\S+) \\(([+-]\\d+)%\\), worst (\\S+) \\(([+-]\\d+)%\\)\\.', '$1. kör: legjobb $2 ($3%), legrosszabb $4 ($5%).'],
    // Replay moments.
    ['Portfolio up (\\d+)% in one round', 'A portfólió $1%-kal nőtt egy kör alatt'],
    ['A millionaire at the close: (.+)', 'Milliomos a záráskor: $1'],
  ],
};

export default hu;
