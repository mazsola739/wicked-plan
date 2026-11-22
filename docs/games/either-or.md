# Either Or

Main game, 3–8 seats (the page: you + 3 bots). Code: `crates/rules/src/games/either_or.rs`.

- **14 rounds:** 10 two-way questions ("Sunrise or sunset?") and **4 surprise mini rounds** dropped in
  between them at random places (never before the first question). Every round is a secret,
  simultaneous pick with **15 seconds**; when time runs out a random answer is picked for you.
- **Question rounds** say their rule up front:
  - **Majority:** everyone on the bigger side scores **+1**.
  - **Minority:** everyone on the smaller side scores **+2**.
  - A tie (or everyone on one side for Minority) scores nothing.
- **Mini rounds** (each appears once, in a random order):
  - **Lowest Unique:** pick 1–5; the lowest number nobody else picked scores **+3**.
  - **Share or Grab:** all share → **+2** each; exactly one grabs → that player **+5**; two or more
    grab → each grabber **−1**.
  - **Point the Finger:** name another player; the single most-named player loses **3**, and everyone
    who named them scores **+1**. A tie for most-named: nothing happens.
  - **Lone Wolf:** a two-way question; a player who is **alone** on their side scores **+4**.
- After each round everyone's pick and the points are shown.
- Results: total score, ranked; points 1000 for first down to 100 for last.
- Moves: `{type:"pick", value}` — 0/1 for two-way rounds, 1–5 for Lowest Unique, 0 share / 1 grab,
  a seat number for Point the Finger.
- Bot: every question has a hidden lean that most bots follow (so a majority forms); in Minority and
  Lone Wolf rounds bots go against the lean more often; it grabs now and then, points at the leader.
- Desktop: 1 / 2 pick the left / right answer (1–5 for numbers, seat letters A–H for players).
