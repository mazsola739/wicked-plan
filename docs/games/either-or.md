# Either Or

Main game, 3–8 seats (the page: you + 3 bots). Code: `crates/rules/src/games/either_or.rs`.

## How to play

**Goal:** score the most points over 14 rounds by guessing how the others will pick.

- Most rounds ask a question with two answers. Everyone picks in secret at the same time. Tap your answer.
- The round tells you its rule first. **Majority:** everyone on the bigger side gets +1. **Minority:** everyone on the smaller side gets +2. A tie scores nothing.
- Four surprise rounds show up once each, between the questions.
- **Lowest Unique:** pick 1–5. The lowest number nobody else picked scores +3.
- **Share or Grab:** if all share, everyone gets +2. If exactly one grabs, they get +5. If two or more grab, each grabber loses 1.
- **Point the Finger:** name another player. The single most-named player loses 3, and everyone who named them gets +1.
- **Lone Wolf:** a two-way question. Anyone alone on their side gets +4.
- After each round you see everyone's picks.

## Rules in detail

- **14 rounds:** 10 two-way questions ("Sunrise or sunset?") and **4 surprise mini rounds** dropped in
  between them at random places (never before the first question). Every round is a secret,
  simultaneous pick.
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
- **Replay moments:** the only grabber in Share or Grab ("Lone grab while the rest shared (+5)"), and a
  player who was behind before the last round and ends it alone in the lead ("Lead stolen in the very
  last round"). Public: every pick and score is shown after each round anyway.
