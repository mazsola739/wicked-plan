// Hungarian for Sniper Hold'em (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

// The moment "{seat}'s <rank> sniped (+n)": one pattern per hand rank (the rank's possessive form).
const SNIPED: [string, string][] = [
  ['high card', 'magas lapja'],
  ['pair', 'párja'],
  ['two pair', 'két párja'],
  ['three of a kind', 'drillje'],
  ['straight', 'sora'],
  ['flush', 'flushe'],
  ['full house', 'fullja'],
  ['four of a kind', 'pókere'],
  ['straight flush', 'színsora'],
];

const hu: Locale = {
  name: 'Mesterlövész-póker',
  short: 'Póker, amelyben „kilőhetsz” egy lapkombinációt, és az kiesik.',
  strings: {
    'High card': 'Magas lap',
    Pair: 'Pár',
    'Two pair': 'Két pár',
    'Three of a kind': 'Drill',
    Straight: 'Sor',
    Flush: 'Flush',
    'Full house': 'Full',
    'Four of a kind': 'Póker',
    'Straight flush': 'Színsor',
    red: 'piros',
    blue: 'kék',
    green: 'zöld',
    gold: 'arany',
    ante: 'beszálló',
    check: 'passz',
    call: 'megad',
    raise: 'emel',
    fold: 'bedob',
    '{n} to call.': '{n} a megadáshoz.',
    'Your turn.': 'Te jössz.',
    '{name} is thinking…': '{name} gondolkodik…',
    'You folded. Waiting for the snipes…': 'Bedobtad. Várakozás a kilövésekre…',
    'Snipe a hand rank, or none.': 'Lőj ki egy lapkombinációt, vagy egyiket se.',
    'Waiting for the other snipes…': 'Várakozás a többi kilövésre…',
    'Every hand was knocked out: {n} chips carry over.': 'Minden kéz kiesett: {n} zseton átkerül a következő leosztásba.',
    'You win {n} chips.': '{n} zsetont nyersz.',
    '{name} wins {n} chips.': '{name} {n} zsetont nyer.',
    '{names} win {n} chips.': '{names} {n} zsetont nyernek.',
    ' and ': ' és ',
    'Hand {n} of {total}': '{n}. leosztás / {total}',
    Pot: 'Kassza',
    '{n} carried over': '{n} áthozva',
    'Everyone else folded.': 'Mindenki más bedobta.',
    'Snipes:': 'Kilövések:',
    chosen: 'választott',
    'aiming…': 'céloz…',
    'bet {n}': 'tét {n}',
    'You have: {rank}': 'Neked van: {rank}',
    Fold: 'Bedobás',
    'Call {n}': 'Megadás {n}',
    Check: 'Passz',
    'Raise +2': 'Emelés +2',
    'No snipe': 'Nincs kilövés',
    'You snipe: {rank}.': 'Kilövésed: {rank}.',
    nothing: 'semmi',
    'C check/call · R raise · F fold · snipe: 1–9 a rank, 0 none.':
      'C passz/megadás · R emelés · F bedobás · kilövés: 1–9 egy kombináció, 0 egyik sem.',
  },
  // Replay moments written by the rules (crates/rules/src/games/sniper_holdem.rs).
  patterns: SNIPED.map(([en, hu]): [string, string] => [`(\\{[a-z0-9-]+\\})'s ${en} sniped \\(\\+(\\d+)\\)`, `$1 ${hu} kilőve (+$2)`]),
};

export default hu;
