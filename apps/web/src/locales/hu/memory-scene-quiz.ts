// Hungarian for Fragments of Memory (i18n.ts). The English text is the key.
// The rules build each question from a shape, a colour and a place (memory_scene_quiz.rs:
// all_questions), so every question and answer they can write is listed here from those words.

import type { Locale } from '../../i18n';

const SHAPES: Record<string, string> = {
  circle: 'kör',
  square: 'négyzet',
  triangle: 'háromszög',
  star: 'csillag',
  heart: 'szív',
  moon: 'hold',
};
const COLOURS: Record<string, string> = { red: 'piros', blue: 'kék', green: 'zöld', yellow: 'sárga', purple: 'lila' };
const ROWS: Record<string, string> = { top: 'felső', middle: 'középső', bottom: 'alsó' };
const COLS: Record<string, string> = { first: 'első', second: 'második', third: 'harmadik', fourth: 'negyedik' };

const built: Record<string, string> = { 'Which of these was NOT in the scene?': 'Melyik NEM volt a képen?' };
for (const [s, hs] of Object.entries(SHAPES)) {
  built[s] = hs;
  built[`What colour was the ${s}?`] = `Milyen színű volt a ${hs}?`;
  built[`How many ${s}s were there in all?`] = `Hány ${hs} volt összesen?`;
  for (const [c, hc] of Object.entries(COLOURS)) {
    built[`${c} ${s}`] = `${hc} ${hs}`;
    built[`How many ${c} ${s}s were in their cell?`] = `Hány ${hc} ${hs} volt a mezőjében?`;
    built[`What was just right of the ${c} ${s}?`] = `Mi volt közvetlenül a ${hc} ${hs} jobb oldalán?`;
  }
}
for (const [c, hc] of Object.entries(COLOURS)) {
  built[c] = hc;
  built[`How many cells held something ${c}?`] = `Hány mezőben volt valami ${hc}?`;
}
for (const [r, hr] of Object.entries(ROWS)) {
  for (const [c, hc] of Object.entries(COLS)) {
    built[`Which shape was in the ${r} row, ${c} from the left?`] = `Melyik alakzat volt a ${hr} sorban, balról a ${hc} helyen?`;
  }
}

const hu: Locale = {
  name: 'Emléktöredékek',
  short: 'Jegyezz meg egy képet, aztán sorban válaszoljatok 10 kérdésre. Egy rossz válasz, és a csapat elbukik.',
  strings: {
    ...built,
    You: 'Te',
    'Scene to remember': 'A megjegyzendő kép',
    'Memorise the scene!': 'Jegyezd meg a képet!',
    'Question {n} of {total}': '{n}. kérdés / {total}',
    'Shapes, colours, how many, and where. Then 10 questions, one wrong answer fails the team.':
      'Alakzatok, színek, hány darab és hol. Utána 10 kérdés jön; egy rossz válasz, és a csapat elbukik.',
    'Waiting for the team…': 'Várakozás a csapatra…',
    Ready: 'Kész',
    '{n} of {total} ready': '{n} / {total} kész',
    'Keys: Enter = Ready.': 'Billentyűk: Enter = Kész.',
    'it was {answer}': 'a jó válasz: {answer}',
    'Your question': 'A te kérdésed',
    '{name} answers this one': 'Ezt {name} válaszolja meg',
    'Keys: 1–4 answer.': 'Billentyűk: 1–4 válaszol.',
  },
  // Replay moment written by the rules (crates/rules/src/games/memory_scene_quiz.rs).
  patterns: [['Question (\\d+) right: a perfect (\\d+)', 'A(z) $1. kérdés is jó: hibátlan $2 pont']],
};

export default hu;
