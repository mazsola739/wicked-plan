# Secret Code

Exile game, a race: 1–4 seats (the page: you + 3 bots). Code: `crates/rules/src/games/secret_code.rs`.

- Under the cup lie **four dice** in a row; the code is their faces (1–6, faces may repeat).
- **Our twist:** the **total of the pips** is shown from the start.
- Everyone races on the **same code**. **3 codes**, **2 minutes** each, at most **8 guesses** per code.
- A guess is four faces. The answer: **exact** (right face in the right place) and **near** (right face,
  wrong place; every die counts once).
- Points for a solved code: **60 + 25 × (9 − guesses used)** (1 guess 260, 8 guesses 85). The fastest
  solver of a code (by time; ties share) gets **+50**. Unsolved: 0.
- When you have solved the code or used all 8 guesses, the rest of the round is played out at once and
  the code is shown for **4 seconds** (the next code starts sooner with Next). Results: `rank_by_score`.
- Moves: `{type:"guess", code:[a,b,c,d]}` (faces 1–6), `{type:"next"}` (during the reveal).
- View: round, timer, the pip total, your guesses with their answers, per rival the number of guesses and
  whether they solved it (never their guesses), scores; the code only in the reveal and at the end.
- Rivals (every seat after the first) are played by the rules with seeded timing: first guess after 4–9 s,
  then one every 5–12 s; 3 in 4 guesses fit every answer so far (and the total), the rest are hunches.
- Bot (for the first seat in tests): a random code that fits every answer and the total.
- Desktop: keys 1–6 add a die, Backspace removes, Enter guesses.
