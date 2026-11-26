// Hungarian for Either Or (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The answers of the two-way questions (crates/rules/src/games/either_or.rs, PROMPTS), also used
// to build the "… wins" lines below.
const OPTIONS: Record<string, string> = {
  Sunrise: 'Napfelkelte',
  Sunset: 'Naplemente',
  Coffee: 'Kávé',
  Tea: 'Tea',
  Mountains: 'Hegyek',
  Seaside: 'Tengerpart',
  Cat: 'Macska',
  Dog: 'Kutya',
  Flying: 'Repülés',
  Invisibility: 'Láthatatlanság',
  City: 'Város',
  Countryside: 'Vidék',
  'Too hot': 'Túl meleg',
  'Too cold': 'Túl hideg',
  Sweet: 'Édes',
  Savoury: 'Sós',
  Fast: 'Gyors',
  Quiet: 'Csendes',
  'A map': 'Egy térkép',
  'Your gut': 'A megérzésed',
  Window: 'Ablak',
  Aisle: 'Folyosó',
  'Early morning': 'Kora reggel',
  'Late night': 'Késő éjjel',
  'Plan ahead': 'Előre tervezek',
  Improvise: 'Rögtönzök',
  'Cash now': 'Pénz most',
  'Double next week': 'Dupla jövő héten',
  Rope: 'Kötél',
  Torch: 'Zseblámpa',
  Roof: 'Tetőn',
  Tunnel: 'Alagúton',
  Share: 'Megosztom',
  Grab: 'Elveszem',
};

const escapeRe = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

const hu: Locale = {
  name: 'Ez vagy az',
  short: 'Tíz kétesélyes választás, ahol a többség vagy a kisebbség pontoz, közben négy meglepetés minikörrel.',
  strings: {
    ...OPTIONS,
    // Round titles (the screen).
    'Majority wins · +1': 'A többség nyer · +1',
    'Minority wins · +2': 'A kisebbség nyer · +2',
    'Surprise: Lowest Unique': 'Meglepetés: Legkisebb egyedi',
    'Surprise: Share or Grab': 'Meglepetés: Megosztod vagy elveszed',
    'Surprise: Point the Finger': 'Meglepetés: Mutass rá',
    'Surprise: Lone Wolf · alone scores +4': 'Meglepetés: Magányos farkas · aki egyedül marad, +4',
    'Game over': 'Vége a játéknak',
    'Round {n} of {total}': '{n}. kör / {total}',
    'Round {n}': '{n}. kör',
    'Waiting for {names}…': 'Várakozás: {names}…',
    'the others': 'a többiek',
    'Keys: 1 / 2 for the answers, 1–5 for numbers, A–H for players.':
      'Billentyűk: 1 / 2 a válaszokhoz, 1–5 a számokhoz, A–H a játékosokhoz.',

    // Questions and mini-round prompts (the rules).
    'Which would you rather keep?': 'Melyiket tartanád meg inkább?',
    'Pick a drink for a long night.': 'Válassz italt egy hosszú éjszakára.',
    'Your dream holiday?': 'Az álomnyaralásod?',
    'The better pet?': 'A jobb háziállat?',
    'Pick a superpower.': 'Válassz szupererőt.',
    'Where do you sleep better?': 'Hol alszol jobban?',
    'Which is worse?': 'Melyik a rosszabb?',
    'Pick a breakfast.': 'Válassz reggelit.',
    'Your getaway car?': 'A menekülőautód?',
    'Which would you trust?': 'Melyikben bíznál?',
    'Pick a seat.': 'Válassz ülést.',
    'Better time to think?': 'Mikor gondolkodsz jobban?',
    'How do you win?': 'Hogyan nyersz?',
    'Pick a prize.': 'Válassz nyereményt.',
    'Which tool?': 'Melyik eszköz?',
    'Escape through the…': 'Merre szöksz?',
    'Lowest Unique: pick 1–5. The lowest number nobody else picked scores +3.':
      'Legkisebb egyedi: válassz 1–5 között. A legkisebb szám, amit senki más nem választott, +3 pontot ér.',
    'Share or Grab: all share, +2 each. One grabber takes +5. Two or more grabbers, −1 each.':
      'Megosztod vagy elveszed: ha mindenki megoszt, fejenként +2. Egyetlen elvevő +5-öt kap. Két vagy több elvevő fejenként −1.',
    'Point the Finger: name a player. The most-named loses 3; who named them scores +1.':
      'Mutass rá: nevezz meg egy játékost. A legtöbbször megnevezett 3 pontot veszít; aki őt nevezte meg, +1.',

    // Round results (the rules).
    'No winning side.': 'Egyik oldal sem nyert.',
    'Nobody stood alone.': 'Senki sem maradt egyedül.',
    'A lone wolf scores +4.': 'A magányos farkas +4 pontot kap.',
    'No unique number.': 'Nincs egyedi szám.',
    'Everyone shared: +2 each.': 'Mindenki megosztott: fejenként +2.',
    'One grabber took +5.': 'Egyetlen elvevő vitte a +5-öt.',
    'Too many grabbers: −1 each.': 'Túl sok elvevő: fejenként −1.',
    'A tie for most named: nothing happens.': 'Holtverseny a legtöbb jelölésnél: nem történik semmi.',

    // Replay moments (the rules).
    'Lone grab while the rest shared (+5)': 'Egyedül vett el, miközben a többiek megosztottak (+5)',
    'Lead stolen in the very last round': 'Az utolsó körben vette át a vezetést',
  },
  patterns: [
    ...Object.entries(OPTIONS).map(([en, hu]): [string, string] => [
      `${escapeRe(en)} wins: \\+(\\d+) each\\.`,
      `${hu} nyert: fejenként +$1.`,
    ]),
    ['(\\d) was the lowest unique number: \\+3\\.', '$1 volt a legkisebb egyedi szám: +3.'],
    ['(\\S+) was named most: −3\\.', 'A legtöbbször megnevezett: $1, −3.'],
  ],
};

export default hu;
