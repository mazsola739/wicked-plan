// Hungarian for Triple Dice (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Hármas kocka',
  short: 'Rakj össze egy háromkockás kezet rejtett kockákkal, aztán tippeld meg a saját helyezésedet.',
  strings: {
    nothing: 'semmi',
    pair: 'pár',
    straight: 'sor',
    triple: 'hármas',
    '1st': '1.',
    '2nd': '2.',
    '3rd': '3.',
    '4th': '4.',
    '5th': '5.',
    '6th': '6.',
    'unknown die': 'ismeretlen kocka',
    'die {n}': '{n}-es kocka',
    'Keep your secret die, or reroll it once?': 'Megtartod a titkos kockádat, vagy egyszer újradobod?',
    'You predict {rank}. Waiting…': 'A tipped: {rank} hely. Várakozás…',
    'Where will your hand rank?': 'Hányadik lesz a kezed?',
    'Your hand: {kind}, ranked {rank} (you said {said}): +{points}.': 'A kezed: {kind}, {rank} hely (tipped: {said}): +{points}.',
    'Round {n} of {total}': '{n}. kör / {total}',
    'said {rank}': 'tipp: {rank}',
    rerolled: 'újradobott',
    ready: 'kész',
    'thinking…': 'gondolkodik…',
    'Triple › straight › pair › nothing; then the higher sum.': 'Hármas › sor › pár › semmi; utána a nagyobb összeg.',
    open: 'nyílt',
    secret: 'titkos',
    blind: 'vak',
    Keep: 'Megtartom',
    'Reroll secret': 'Titkos újradobása',
    'K keep · R reroll · 1–{n} predict your rank.': 'K megtartás · R újradobás · 1–{n} tippeld a helyezésed.',
  },
  // Replay moments written by the rules (crates/rules/src/games/dice_prediction.rs).
  patterns: [
    ['Triple (\\d)s, rank (\\d+) called', 'Hármas ($1 $1 $1), a(z) $2. hely eltalálva'],
    ['(\\d+) exact rank calls in a row', '$1 pontos helyezéstipp zsinórban'],
  ],
};

export default hu;
