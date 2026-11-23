# Jewel Black Market

Main game, 4–8 seats (the page: you + 3 bots). Code: `crates/rules/src/games/jewel_black_market.rs`.

## How to play

**Goal:** score the most over 3 rounds by ending each round with a good card.

- Each round you get one hidden card. Three cards lie face down in the buyer's space and one in the discard.
- Jewels: Diamond 5 and 6, Gold 4 and 5, Silver 2, 3, 4, Bronze 1, 1, 2, 2. Bombs: −6 and −4.
- There are two swap rounds. On your turn (20 seconds), tap **Keep my card**, tap a player to swap with them, or tap a buyer card or the **Discard** to swap with it.
- If you hold a bomb, you must swap it with a player.
- At the end of the round a jewel scores its value times the number of same-type jewels in the buyer's space (none there: 0). A bomb costs its value.
- Others only see who swapped with whom, so remember what you saw.
- If time runs out, you keep your card, and a bomb goes to the next seat.

## Rules in detail

- **13 cards:** Diamond 5, 6 · Gold 4, 5 · Silver 2, 3, 4 · Bronze 1, 1, 2, 2 · Bomb −6, −4.
- **3 rounds.** Each round the cards are shuffled: every seat gets **one hidden card**, **3 go face down
  to the buyer's space**, **1 face down to the discard**; the rest are set aside unseen.
- **Two swap rounds:** first in seat order, then in reverse order (the starting seat moves one place
  each round). On your turn, within **20 seconds**:
  - **swap with another player** (you both see your new cards; the table only sees who swapped with whom),
  - **swap with one of the buyer's 3 cards** (you see the card you take),
  - **swap with the discard**,
  - or **keep** your card.
  A **bomb holder must swap with a player** (no keep, no buyer, no discard).
  When time runs out: keep (a bomb goes to the next seat).
- **Scoring** at the end of the round: a jewel scores **its value × the number of jewels of the same type
  in the buyer's space** (none there: 0). A bomb scores its own value (−6 or −4). Bombs in the
  buyer's space or the discard do nothing.
- Every card is shown after the round (buyer's space, discard, each hand, points).
- Results: total over 3 rounds, ranked; points 1000 for first down to 100 for last.
- Moves: `{type:"keep"}`, `{type:"swapPlayer", seat}`, `{type:"swapBuyer", slot}` (0–2), `{type:"swapDiscard"}`.
- View: your card, whose turn, the round's public log plus what you saw, scores, the last round's reveal.
- Bot: a bomb goes to a random player; a jewel worth 4+ is usually kept; a weaker card is swapped
  with the buyer, a player or the discard.
- Desktop: K keeps, 1–3 swap with a buyer card, D the discard, the seat letters A–H a player.
