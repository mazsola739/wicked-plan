# Auction Bingo

Main game, 3–8 seats (the page: you + 3 bots). Code: `crates/rules/src/games/auction_bingo.rs`.

## How to play

**Goal:** mark the most lines on your secret 3×3 bingo board.

- Your board holds 3 circles, 3 triangles and 3 stars. You start with 20 coins.
- There are 7 rounds. First tap an empty cell (20 seconds): you go for its shape this round.
- You then see how many players picked each shape, but not who.
- Next set a bid and tap **Bid** (20 seconds). Bids are secret.
- For each shape, the single highest bidder among those who picked it pays and marks the cell. A tie: nobody wins that shape. Losers keep their coins.
- No pick means a random empty cell; no bid means 0.
- You score 5 per completed line (rows, columns, diagonals), 1 per marked cell and 1 per 5 coins left.
- Tip: if few others picked your shape, a small bid may be enough.

## Rules in detail

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
- **Replay moments:** a won cell that completes a line (or two at once) while at least one rival
  picked the same shape ("Bingo: 2 rivals outbid (+5)"). Only the winner sees it: who won and
  every board stay secret until the end; the number of rivals per shape is public.
