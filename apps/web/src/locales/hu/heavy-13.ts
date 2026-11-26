// Hungarian for Heavy 13 (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Nehéz 13',
  short: 'Számkártyák, szivárványzseton és passzok: színes köveket raksz kupacokba; a túlterhelt kupac pontokba kerül.',
  strings: {
    red: 'piros',
    blue: 'kék',
    green: 'zöld',
    gold: 'arany',
    passed: 'passz',
    rainbow: 'szivárvány',
    'You {play}. Waiting…': 'Te: {play}. Várakozás…',
    'Pick a card, or pass.': 'Válassz egy kártyát, vagy passzolj.',
    'Tap a stack to play it there.': 'Koppints egy kupacra, hogy oda játszd ki.',
    'You {play}: {points}.': 'Te: {play}: {points}.',
    'Round {n} of {total}': '{n}. kör / {total}',
    full: 'telt',
    overloaded: 'túlterhelt',
    '{n} left': 'még {n}',
    '{n} cards · {p} passes': '{n} kártya · {p} passz',
    chip: 'zseton',
    ready: 'kész',
    'thinking…': 'gondolkodik…',
    'Pass ({n} left)': 'Passz (még {n})',
    'Play on its colour': 'A saját színére',
    'Rainbow chip ready: tap another stack to use it.': 'A szivárványzseton kész: koppints egy másik kupacra, hogy felhasználd.',
    'Rainbow chip used.': 'A szivárványzseton elfogyott.',
    '1–8 pick a card · Q W E R stack · Enter its own stack · P pass.':
      '1–8 kártyaválasztás · Q W E R kupac · Enter a saját kupaca · P passz.',
  },
  // Replay moments written by the rules (crates/rules/src/games/heavy_13.rs).
  patterns: [
    ['Rainbow (\\d) fills red to exactly 13', 'Szivárványos $1 pontosan 13-ra tölti a pirosat'],
    ['Rainbow (\\d) fills blue to exactly 13', 'Szivárványos $1 pontosan 13-ra tölti a kéket'],
    ['Rainbow (\\d) fills green to exactly 13', 'Szivárványos $1 pontosan 13-ra tölti a zöldet'],
    ['Rainbow (\\d) fills gold to exactly 13', 'Szivárványos $1 pontosan 13-ra tölti az aranyat'],
    ['Lead stolen in the very last round', 'Vezetés elorozva a legutolsó körben'],
  ],
};

export default hu;
