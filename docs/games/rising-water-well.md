# Rising Water Well

Hidden game, solo. Code: `crates/rules/src/games/rising_water_well.rs`.

## How to play

**Goal:** solve the four riddles on the wall and climb out before the well fills with water.

- The well fills in 10 minutes. Each solved riddle opens one of the four locks on the grate.
- The riddles come one at a time: count shapes, continue tally marks, balance scales, add up symbols, or read a clock seen in the water.
- Every answer is a whole number. Type it on the keypad and tap **OK**.
- A wrong answer lets in more water: you lose 30 seconds.
- Each solved riddle scores 125. Escaping adds 200, plus 1 point for every 2 seconds left.
- If the well fills, you keep the points of the riddles you solved. **Give up** stops and keeps them too.

## Rules in detail

You wake at the bottom of a dry well; water starts to pour in. Four picture riddles are carved into the
wall. Each answer opens one lock of the grate above; open all four to climb out before the well is full.

- **Time:** the well fills in **10 minutes**. A wrong answer lets in a gush of water: **−30 seconds**.
- **Four riddles**, one after the other, drawn from five kinds (which four, and their order, come from the seed):
  1. **Stones** — a pile of shapes in three colours; "How many blue triangles?" (or: how many triangles of
     any colour).
  2. **Scratches** — four groups of tally marks follow a rule (add the same number, add one more each time,
     or double); how many marks come next?
  3. **Scales** — two balanced scales: some circles weigh as much as one square; one triangle weighs as much
     as one square and some circles. How many circles weigh as much as one triangle?
  4. **Symbols** — three sums of symbols with their totals, then a fourth sum to work out.
  5. **Reflection** — a clock seen upside down in the water (the 12 mark at the bottom, the minute hand
     pointing down). What hour does the real clock show (1–12)?
- Answers are whole numbers (0–99), typed on a keypad.
- **Points:** 125 per riddle solved. Escaping (all four) adds **200**, plus 1 point for every 2 seconds left.
  Most: about 1,000. When the well fills, you keep the points of the riddles you solved.
- Moves: `{type:"answer", value}`, `{type:"giveUp"}` (stop and keep the points so far).
- View: `riddle` (the current one, without its answer: `kind` plus what is drawn), `index` (1–4),
  `riddles` (4), `solved`, `wrongTries`, `lastWrong`, `score`, `timeLeftMs`, `totalMs`, `escaped`, `over`.
- Bot: answers right four times out of five, otherwise one too many.
- Desktop: type the number and press Enter.
- **Replay moments:** an escape without a single wrong answer ("Flawless escape, 8:45 to spare"), or
  else an escape with less than a minute left ("Out of the well with 0:42 left"). Solo: only the player
  sees them, and they tell nothing the screen does not.
