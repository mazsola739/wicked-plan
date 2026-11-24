# Memory Poker

Mini game, 1–4 seats (the page: you + 3 bots). Code: `crates/rules/src/games/memory_poker.rs`.

## How to play

**Goal:** remember where the good cards are and take two that make the best poker hand.

- The game has **3 rounds**. Each round, 16 cards lie face up in a 4×4 grid, with **3 shared cards** beside it.
- Study the grid as long as you like, then tap **Ready**. When everyone is ready, the grid turns face down.
- Players take turns picking, in snake order, until everyone has **2 cards**. On your turn, tap a face-down card.
- Only you see the cards you take.
- Your hand is your 2 cards plus the 3 shared cards. A high card scores 20 and a straight flush scores 400, with the usual poker hands in between.
- The best hand of the round gets **+60** more points.
- Tip: watch which squares the others take. Your remembered card may already be gone.

## Rules in detail

- **3 rounds.** Each round 16 cards from a fresh 52-card deck lie in a 4×4 grid, and **3 shared cards**
  lie face up beside it.
- **Study:** the grid is shown face up until every player taps Ready. Then the grid turns face down.
- **Pick:** in snake order (1-2-3-4-4-3-2-1; the first seat moves on each round) every player takes
  **2 cards** from the grid. Only the taker sees a taken card; the others see that its square is empty.
- **Showdown (6 s):** every hand is the 2 taken cards + the 3 shared cards. Points by hand:
  high card 20, pair 50, two pair 90, three of a kind 130, straight 180, flush 200, full house 250,
  four of a kind 320, straight flush 400. The **best hand** of the round (ties share) gets **+60**.
  Aces count high, and low in A-2-3-4-5. Results: `rank_by_score`.
- Moves: `{type:"ready"}` (study), `{type:"pick", cell}` (0–15), `{type:"next"}` (ends the showdown early).
- View: phase, round, the showdown timer, the grid (faces during study and showdown; afterwards only
  your own taken cards), who took each square, the shared cards, every seat's points and, at the
  showdown, their cards and hand.
- Bot: remembers each grid card with 60% (fixed per round and seat); takes the remembered card that
  makes the best hand with the shared cards and its other pick, or a random free square when it
  remembers none.
- Desktop: keys A–P pick a square (row by row), Enter is Ready / Next.
- **Replay moments:** a straight or better at the showdown ("Straight from memory (+240)", "Full house
  from memory (+250)"). Public: every hand is shown at the showdown, never before.
