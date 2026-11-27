# Mystery Stock Market

Main game, 3–8 players (the page seats you and 3 bots). Code: `crates/rules/src/games/mystery_stock_market.rs`.

## How to play

**Goal:** be the richest player after **24 rounds**, counting your cash and the shares you hold.

- You start with **100,000** in cash. All seven stocks start at a price of **10,000**.
- Each stock moves by its own **secret rule**, one of ten possible rules.
- At the start you get one private clue: the exact rule of one stock. After rounds 8 and 16, one more stock's rule is shown to everyone.
- **Trade:** tap a stock, choose 1, 5 or 10, then tap **Buy** or **Sell**. You can trade as often as you like. You can't borrow money or sell shares you don't have.
- Tap **Done for this round** when you have finished. Then all the prices change.
- Watch the price charts and the public totals of how many shares were bought and sold to work out each stock's rule.
- At the end your worth is your cash plus your shares at the final prices.

**On the show:** every player also started with 100,000, and there were seven made-up stocks too. Every 5 rounds a few more words of each secret rule appeared, until all were complete by round 20 (here one whole rule is shown to everyone after rounds 8 and 16). Player A secretly had to name one player and see them knocked out, so Player A falsely told Player B that finishing last was the only way for Player B to survive. Player B then lowered their own total over several rounds, finished last and was out of the game.

## Rules in detail

Trade seven made-up stocks over 24 rounds. Every stock moves by its own **secret rule**; work the
rules out from the price charts and trade on what you know. The richest player wins.

- **Replay moments:** a player's worth rising **30% or more** in one round ("Portfolio up 40% in one
  round"), shown **only to that player**, since holdings and cash are secret; and the richest player at
  the close when they hold a million or more ("A millionaire at the close: 1,376,000"), public, since
  every worth is shown at the close.

## Setup

- Seven stocks: Lumen Labs, Cobalt Rail, Pebble Foods, Vortex Media, Quill Paper, Ember Energy,
  Tidal Ships. Every price starts at **10,000**; a price never falls below **100**.
- Everyone starts with **100,000** in cash and no shares.
- Each stock gets a different rule, drawn from these ten:

| Rule | Change at the end of a round |
|---|---|
| Steady | +5% |
| Seesaw | +20% after odd rounds, −15% after even rounds |
| Crowd | +4% for every share bought this round by all players together, −4% for every share sold (at most ±40%) |
| Contrarian | the opposite of Crowd |
| Shadow | the same change as one other (named) stock this round |
| Mirror | the opposite change of one other (named) stock this round |
| Bubble | +12% a round until a secret round between 12 and 18, then −60% once, then −5% a round |
| Dice | a random whole number from −25% to +25% |
| Lonely | −10% when more than half the players hold it at the end of the round, +12% otherwise |
| Calendar | +30% after rounds 4, 8, 12, …, −6% after the others |

- Shadow and Mirror follow a stock with one of the other rules. A new price is rounded to whole numbers.
- **Clues:** every player gets one private clue at the start (one stock's exact rule; different stocks
  for different players while possible). After rounds 8 and 16 one more stock's rule is shown to all.

## A round (24 rounds)

- **Trade:** buy and sell any number of shares at the current prices, as often as you like (no
  borrowing, no selling shares you do not have). Tap *Done* when finished.
- When everyone is done, prices change by their rules.
- Public after each round: the new prices, how many shares of each stock were bought minus sold in
  total, and how many players hold each stock. Holdings and cash of others stay secret.

## End and scoring

- After round 24 your worth is your cash plus your shares at the final prices.
- Then every rule is shown (**20 s**, or tap *See results*).
- Rank by worth; points by rank: 1000 for first down to 100 for last, evenly spaced.

## Moves

- `{type:"trade", stock, qty}` (`qty` > 0 buys, < 0 sells), `{type:"done"}`, `{type:"close"}`.

## Bot

For every stock it keeps the rules (with their hidden details) that fit the whole public history
within a percentage point, uses its own clue and the public clues, and expects the average change of
the rules left. It sells what it expects to fall and puts most of its cash in the stock it expects to
rise most (if at least +3%).

Desktop: 1–7 pick a stock, B buys one, S sells one, Enter is Done.
