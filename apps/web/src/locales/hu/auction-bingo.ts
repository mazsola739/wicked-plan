// Hungarian for Auction Bingo (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The rules name the shapes in their log lines; one pattern set per shape.
// [English, Hungarian, Hungarian with the article and the accusative ending]
const SHAPES: [string, string, string][] = [
  ['circle', 'karika', 'a karikát'],
  ['triangle', 'háromszög', 'a háromszöget'],
  ['star', 'csillag', 'a csillagot'],
];

const hu: Locale = {
  name: 'Árverési bingó',
  short: 'Titkos 3×3-as tábla alakzatokkal. Minden körben válassz egy alakzatot, licitálj rá érmékkel, és teljesíts sorokat.',
  strings: {
    circle: 'karika',
    triangle: 'háromszög',
    star: 'csillag',
    All: 'Mind',
    'Bid {n} coin': 'Licit: {n} érme',
    'Bid {n} coins': 'Licit: {n} érme',
    'Game over': 'Vége a játéknak',
    'Round {n} of {total}': '{n}. kör / {total}',
    pick: 'választás',
    bid: 'licit',
    'Coins:': 'Érmék:',
    '5 per line, 1 per cell, 1 per 5 coins left': 'soronként 5, mezőnként 1, minden megmaradt 5 érméért 1',
    'Pick an empty cell: you go for its shape.': 'Válassz egy üres mezőt: annak az alakzatára mész.',
    '{n} picked': '{n} választotta',
    ' (you)': ' (köztük te)',
    'Waiting for {names}…': 'Várakozás rájuk: {names}…',
    'Final boards': 'Végső táblák',
    'Keys: 1–9 pick a cell; type a bid and press Enter.': 'Billentyűk: 1–9 választ egy mezőt; írd be a licitet, és nyomj Entert.',
  },
  // Log lines and replay moments written by the rules (crates/rules/src/games/auction_bingo.rs).
  patterns: [
    ['Round (\\d+):', '$1. kör:'],
    ...SHAPES.flatMap(([en, hu, acc]): [string, string][] => [
      [`${en}: nobody picked it\\.`, `${hu}: senki sem választotta.`],
      [`${en}: tie at (\\d+), nobody won\\.`, `${hu}: holtverseny ($1 érme), senki sem nyert.`],
      [`${en}: won for (\\d+) coins?\\.`, `${hu}: elkelt $1 érméért.`],
      [`You did not win the ${en}\\.`, `Nem nyerted el ${acc}.`],
      [`You won the ${en} for (\\d+)\\.`, `Elnyerted ${acc} $1 érméért.`],
    ]),
    ['Bingo: (\\d+) rivals? outbid \\(\\+(\\d+)\\)', 'Bingó: $1 ellenfél túllicitálva (+$2)'],
    ['(\\d+) lines at once: (\\d+) rivals? outbid \\(\\+(\\d+)\\)', 'Egyszerre $1 sor: $2 ellenfél túllicitálva (+$3)'],
  ],
};

export default hu;
