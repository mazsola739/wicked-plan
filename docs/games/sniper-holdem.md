# Sniper Hold'em

Prison game, 2–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/sniper_holdem.rs`.

## How to play

**Goal:** finish 6 hands of poker with the most chips.

- You start with 25 chips and pay 1 chip each hand. You get 2 private cards, and 4 shared cards lie face up. Your hand is the best five of your six cards.
- There is one betting round: **Check** or **Call**, **Raise +2**, or **Fold**.
- Then everyone still in secretly snipes one hand rank, or taps **No snipe**.
- Every hand whose rank anyone sniped is knocked out, even by your own snipe. The best hand left wins the pot.
- If every hand is knocked out, the pot carries over to the next hand.
- Tip: snipe the rank you fear will beat you, not your own.

## Rules in detail

- **Deck:** 40 cards, values **1–10** in four colours (red, blue, green, gold).
- Everyone starts with **25 chips**. **6 hands**; the game ends earlier when only one player has chips.
- **Each hand:** everyone with chips pays an **ante of 1**. Each gets **2 private cards**; **4 shared cards**
  are dealt face up at once. Your hand is the best five of your six cards.
- **Hand ranks** (low to high): high card, pair, two pair, three of a kind, straight (five values in a row,
  1 is only low), flush (five of one colour), full house, four of a kind, straight flush. Equal ranks
  compare the deciding values as in ordinary poker.
- **Betting:** one round, starting left of the dealer button: **check / call**, **raise**
  (+2 chips) or **fold**. At most 3 raises a hand. **Table limit:** nobody may bet more in a hand than the
  smallest stack among the players in it had after the ante, so there are never side pots. If everyone
  else folds, the last player takes the pot.
- **Snipe:** everyone still in secretly names **one hand rank, or none**. Then the snipes are
  revealed together.
- **Showdown:** every hand whose rank was sniped by anyone (your own snipe included) is **knocked out**.
  The best hand that survives wins the pot; ties split it (the odd chip goes to the first in seat order).
  If every hand is knocked out, the pot **carries over** to the next hand.
- **Result (6 s)**, then the button moves on.
- **Ranking:** most chips (a pot still carried over at the end is lost). Points by rank: 1000 for the first,
  100 for the last, evenly between.
- **View:** your cards, the shared cards, the pot, everyone's chips, bet and last action; at the showdown
  the hands still in, their ranks, the snipes and who was knocked out.
- **Moves:** `{type:"call"}` (also checks), `{type:"raise"}`, `{type:"fold"}`, `{type:"snipe", rank}` with
  `rank` one of `highCard pair twoPair trips straight flush fullHouse quads straightFlush` or `null`.
- **Bot:** estimates its chance to win by dealing the unseen cards to imaginary opponents; raises with a
  strong hand, calls with a fair one, folds a weak one against a bet, and bluffs a raise now and then.
  It snipes the rank that most often beats its own hand in those samples (never its own rank).
- Desktop: C check/call, R raise, F fold; in the snipe step 1–9 pick a rank, 0 for none.
- **Replay moments:** at the showdown, a winner who sniped the rank of the best hand at the table, for a
  pot of 10 chips or more ("{bot-2}'s straight sniped (+12)"). Public: the showdown shows every hand
  still in, its rank and every snipe anyway; folded hands never come into it.
