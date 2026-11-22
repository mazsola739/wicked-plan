# Auction Bingo

Main game, 3–8 seats (the page: you + 3 bots). Code: `crates/rules/src/games/auction_bingo.rs`.

- Everyone has their own **3×3 bingo board**: 3 circles, 3 triangles and 3 stars in a random layout,
  and **20 coins**. Boards and coins are secret.
- **7 rounds**, each in two secret, simultaneous steps:
  1. **Pick** (20 s): choose one empty cell of your board; its shape is what you go for this round.
     Then the table sees **how many players picked each shape** (not who).
  2. **Bid** (20 s): bid 0 up to all your coins.
- Per shape, the **single highest bidder** among the players who picked it **wins**: they pay their bid
  and mark their cell. A tie for the highest bid: nobody wins that shape. Everyone else keeps their coins.
- Who won stays secret: the table only hears the winning bid per shape (or "tie" / "nobody").
  You learn whether you won.
- Timeouts: no pick → a random empty cell; no bid → 0.
- **Score:** 5 per completed line (rows, columns, diagonals), 1 per marked cell, 1 per 5 coins left.
  All boards are shown at the end. Points: 1000 for first down to 100 for last.
- Moves: `{type:"pick", cell}` (0–8), `{type:"bid", coins}`.
- Bot: picks the cell that builds the most lines; bids little when it is alone on its shape,
  more when others want it too, pacing its coins over the rounds left.
- Desktop: 1–9 pick a cell, digits + Enter bid.
