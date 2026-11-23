# Weight Balance Puzzle

Team mini game, 1–2 players (on the page: you + 1 bot teammate). Code:
`crates/rules/src/games/weight_balance_puzzle.rs`.

## How to play

**Goal:** work out the secret weights of five blocks, A–E, together with your teammate.

- The weights are five different whole numbers from 1 to 9 grams.
- You share one scale and **12 weighings**, taking turns. Every result is shown to both of you.
- On your turn, tap blocks to move them off, onto the left pan or onto the right pan. Add a counterweight of 0–20 g per pan if you like, then tap **Weigh**.
- The scale only shows which side is heavier, or that it balances. You have **40 seconds**, or that weighing is lost.
- When you are sure, tap **Answer now**. Then each of you writes all five weights in secret, within **120 seconds**.
- The team gets **150 points** for each block you both got right. All five right: **+50 for every unused weighing**.

## Rules in detail

- **Five blocks** A–E have secret weights: five **different whole numbers from 1 to 9** grams.
- **One shared scale, 12 weighings** for the whole team. Players take turns: on your turn you load the two
  pans with any blocks (each block once) plus a counterweight of **0–20 g** per pan, and the scale shows
  **left heavier, right heavier or balanced**. Every weighing is shown to the whole team.
- **Turn timer:** 40 seconds; if it runs out, that weighing is lost.
- On your turn you may instead call **"Answer now"**: weighing stops for the whole team.
- **Answer:** when the weighings run out or someone calls "Answer now", every player writes down a weight for
  each block, **in secret** (separate rooms): 120 seconds. Not sent in time: counts as wrong.
- **Score (team):** **150 points** for every block that **every** player got right. If all five are right,
  **+50 for every unused weighing**. Every player gets the team's points and shares rank 1.
- **View:** the weighing log, whose turn it is, your own answer and who has answered. Weights and the
  teammates' answers are shown only at the end.
- **Moves:** `{type:"weigh", left:[i…], right:[i…], leftExtra, rightExtra}` (blocks 0–4),
  `{type:"answer"}` (stop weighing), `{type:"submit", weights:[w0…w4]}`.
- **Bot:** keeps the list of weight sets that fit every weighing so far and picks the weighing whose worst
  result leaves the fewest; calls "Answer now" as soon as only one set fits, and answers with a set that fits.
- **Screen:** the scale drawn in SVG; tap blocks to cycle them between off / left pan / right pan, steppers
  for the counterweights. Desktop keys: A–E cycle a block, Enter weighs.
- **Replay moments:** at the end, all five weights right for everyone with 3 or more weighings unused,
  credited to whoever made the last weighing ("All five weights right, 3 weighings to spare"). Public:
  the weights and every answer are shown at the end.
