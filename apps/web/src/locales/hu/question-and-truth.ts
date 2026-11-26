// Hungarian for Question and Truth (i18n.ts). The English text is the key.

import type { Locale } from '../../i18n';

const hu: Locale = {
  name: 'Kérdés és igazság',
  short: 'Költs tokeneket kérdésekre, hogy kitaláld, milyen sorrendben áll az ellenfeled nyolc titkos kártyája.',
  strings: {
    'Card at…': 'Kártya itt…',
    'Where is…': 'Hol van…',
    'Sum of two': 'Két hely összege',
    'Left of?': 'Balrább?',
    'Bigger?': 'Nagyobb?',
    'Left half?': 'Bal fél?',
    position: 'hely',
    card: 'kártya',
    'Which card is at position {a}?': '{a}. hely: melyik kártya?',
    'Where is card {a}?': 'Kártya {a}: hányadik helyen van?',
    'Sum of positions {a} and {b}?': '{a}. és {b}. hely összege?',
    'Is card {a} left of card {b}?': 'Kártya {a} balrább van, mint kártya {b}?',
    'Is position {a} bigger than {b}?': '{a}. hely nagyobb, mint {b}. hely?',
    'Is card {a} in the left half?': 'Kártya {a} a bal felében van?',
    'You have guessed.': 'Tippeltél.',
    '{name} goes on…': '{name} folytatja…',
    'Your turn: ask a question or make your final guess.': 'Te jössz: kérdezz, vagy add le a végső tipped.',
    '{name} is asking…': '{name} kérdez…',
    'Tokens: {name} {n}': 'Tokenek: {name} {n}',
    guessed: 'tippelt',
    'Your secret row': 'A titkos sorod',
    "Your guess of {name}'s row (tap a slot, then a card)": 'Tipped {name} sorára (koppints egy helyre, aztán egy kártyára)',
    'Your questions': 'A kérdéseid',
    'No questions yet.': 'Még nincs kérdés.',
    "{name}'s questions about your row": '{name} kérdései a sorodról',
    'None yet.': 'Még egy sincs.',
    Ask: 'Kérdezek',
    '{n} token': '{n} token',
    '{n} tokens': '{n} token',
    'Tap again: no more questions after this': 'Koppints újra: ezután nincs több kérdés',
    'Final guess': 'Végső tipp',
    'Keys: 1–8 fill the guess slot, ← → move, Backspace clears.': 'Billentyűk: 1–8 kitölti a helyet, ← → léptet, a Backspace töröl.',
    Yes: 'Igen',
    No: 'Nem',
  },
  // Questions, answers and replay moments written by the rules (crates/rules/src/games/question_and_truth.rs).
  patterns: [
    ['Which card is at position (\\d+)\\?', '$1. hely: melyik kártya?'],
    ['Where is card (\\d+)\\?', 'Kártya $1: hányadik helyen van?'],
    ['Sum of the cards at positions (\\d+) and (\\d+)\\?', '$1. és $2. hely kártyáinak összege?'],
    ['Is card (\\d+) left of card (\\d+)\\?', 'Kártya $1 balrább van, mint kártya $2?'],
    ['Is the card at position (\\d+) bigger than at position (\\d+)\\?', '$1. hely kártyája nagyobb, mint $2. helyé?'],
    ['Is card (\\d+) in the left half\\?', 'Kártya $1 a bal felében van?'],
    ['Position (\\d+)', '$1. hely'],
    ['Card (\\d+)', 'Kártya $1'],
    ['Row cracked with (\\d+) tokens? to spare', 'Sor feltörve, $1 token maradt'],
    ['Lucky guess: the right row of (\\d+) left', 'Szerencsés tipp: a helyes sor $1 lehetséges közül'],
  ],
};

export default hu;
