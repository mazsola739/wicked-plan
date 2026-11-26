// Hungarian for Treasure Island (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Kincses sziget',
  short: 'Licitálj titokban a nyomokat rejtő ládákra, amelyek nyilai a kincs felé mutatnak, aztán áss egyszer.',
  strings: {
    'Island map': 'A sziget térképe',
    'Box {n} is up. Your secret bid:': 'A(z) {n}. láda van soron. A titkos licited:',
    'Bid sent, waiting…': 'Licit elküldve, várakozás…',
    'Bid {n} coins': 'Licit: {n} érme',
    'Keys: ↑/↓ change the bid by 1, PgUp/PgDn by 10, Enter bids.':
      'Billentyűk: ↑/↓ 1-gyel, PgUp/PgDn 10-zel változtatja a licitet, az Enter licitál.',
    'Auction {n} of {total}': '{n}. árverés / {total}',
    'Dig for the treasure': 'Áss a kincsért',
    'Lit squares still fit your arrows ({n}). Your private box has a cyan rim.':
      'A kivilágított mezők még illenek a nyilaidhoz ({n}). A saját ládádnak türkiz kerete van.',
    'Dug. Waiting for the others…': 'Kiástad. Várakozás a többiekre…',
    'Tap a square, then dig. Only once!': 'Koppints egy mezőre, aztán áss. Csak egyszer!',
    'Dig at {square}': 'Ásás itt: {square}',
    'Pick a square': 'Válassz mezőt',
    'Keys: arrows move the spade, Enter digs.': 'Billentyűk: a nyilak mozgatják az ásót, az Enter ás.',
    'Box {n}:': '{n}. láda:',
    '{names} paid {price}': '{names} fizetett: {price}',
    'nobody bid': 'senki sem licitált',
    '{n} coins': '{n} érme',
    'Dig points: exact 500, one step off 300, two 150, three 50; plus the coins you keep.':
      'Ásáspontok: telitalálat 500, egy lépésre 300, kettőre 150, háromra 50; plusz a megtartott érméid.',
    // Replay moments written by the rules (crates/rules/src/games/treasure_island.rs).
    'Treasure dug up with just the private clue (+500)': 'Kincs kiásva csupán a saját nyommal (+500)',
    'The only dig right on the treasure (+500)': 'Az egyetlen ásás pont a kincsen (+500)',
  },
};

export default hu;
