# Unknown Language

Exile game, solo. Code: `crates/rules/src/games/unknown_language.rs`.

## How to play

**Goal:** crack a picture script from a few translated carvings, then read and write it.

- Each word is a root glyph plus a small mark. The mark tells you if the word is a thing, an action or a quality.
- The same root with another mark is a related word, like "fire", "burns" and "hot".
- Study the example carvings: they show the word order and which mark means what.
- There are **9 questions** in three stages. Each one asks you to pick the right translation or the right script out of 4.
- Stage 1 uses things and actions, stage 2 adds qualities, stage 3 shows known roots with a new mark.
- You have **75 seconds** per question. A right answer is worth 50, plus up to 50 for speed. Wrong or too slow: 0.
- Watch for near misses: swapped subject and object, a changed word, or another word order. (Keys 1–4 on desktop.)

## Rules in detail

The walls of the hidden place are covered in a picture script nobody can read. Some carvings come with a
translation. Work out the script from those examples, then read and write it.

## The script (made from the seed)

- Every word is a **root glyph** (three strokes on a 3 × 3 grid of points) plus a **mark** that says the word
  is a **thing** (noun), an **action** (verb) or a **quality** (adjective). The three marks are a bar under the
  glyph, a dot above it and a ring on its left; which mark means what is chosen by the seed.
- The same root with another mark is a related word: the root *fire* is "fire" as a thing, "burns" as an
  action and "hot" as a quality. Sixteen roots exist (guard, key, fire, water, eye, hand, stone, lamp, bread,
  shadow, door, singer, gold, dog, rope, ice).
- A sentence is *subject*, *action*, *object*, in one of the six possible orders (seeded). A quality stands
  either before or after its thing (seeded). The script has no "the".

## Play

- **Three stages, three questions each** (9 questions), **75 seconds** per question.
  1. Four example sentences with things and actions. Questions: read a sentence (pick the translation
     out of 4) or write one (pick the script out of 4).
  2. Three more examples, now with qualities. Questions as before, with qualities.
  3. No new examples. Two questions show a single word whose root you know but with a new mark: what does
     it mean (4 choices)? The third is a sentence that uses such a word.
- The wrong choices are near misses: subject and object swapped, the words in another order, one word
  changed, or the root read with its old mark.
- **Points** per right answer: 50, plus up to 50 for speed (time left ÷ 75 s). Wrong or out of time: 0.
  Most: 900.
- Moves: `{type:"answer", option}` (0-based).
- View: `stage`, `question` (1–9), `questions`, `examples` (each: `script` words and `english`), `current`
  (`kind`: `read` with `script` + `options` text, `write` with `english` + `options` scripts, or `word` with
  `word` + `options` text), `last` (the previous question, your choice, the right one, points), `score`,
  `timeLeftMs`, `questionTimeMs`, `over`. A script word is `{strokes: [[x1,y1,x2,y2]…], mark}`; the view
  never says which mark means what.
- Bot: right three times out of four.
- Desktop: keys 1–4 pick an option.
