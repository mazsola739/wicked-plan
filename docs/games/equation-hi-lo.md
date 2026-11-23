# Equation Hi-Lo

Main game, 3–6 seats (the page: you + 3 bots). Code: `crates/rules/src/games/equation_hi_lo.rs`.

## How to play

**Goal:** end the game with the most chips.

- Everyone starts with 20 chips. There are 3 hands, and joining a hand costs 1 chip.
- You get 4 number cards: 2 face up that everyone sees, and 2 face down that only you see.
- Bet in turn: **Check** (or **Call**), **Raise 2**, or **Fold**. You have 20 seconds. If you wait too long, you check, or fold if there is something to call.
- Then build an equation in secret within 60 seconds. Tap all 4 cards in any order, with 3 of the 4 symbols + − × ÷ between them, each symbol once.
- × and ÷ are worked out before + and −. Dividing by 0 makes the equation invalid.
- Choose **Low · 1** or **High · 20**, then tap **Submit**. If time runs out, your best equation is built for you.
- A second betting round follows, then the showdown. Half the pot goes to the Low player closest to 1, half to the High player closest to 20. If nobody picked a side, the other side takes it all.

## Rules in detail

- Everyone starts with **20 chips**. **3 hands** (fewer if only one player has chips left).
- **Each hand:** every player with chips pays an **ante of 1**. From a deck of 44 number cards
  (0–10, four of each) everyone gets **4 cards: 2 face up** (everyone sees them) and **2 face down**
  (only you). Everyone also holds the four symbols **+ − × ÷**.
- **Betting round 1**, in turn from the seat after the dealer (the dealer moves on each hand), 20 s each:
  **call** (or check when there is nothing to call), **raise by 2**, or **fold**. At most 2 raises
  per betting round. Nobody can be asked for more chips than the poorest player in the hand had at its start
  (so there is no all-in). Timeout: check, or fold if there is something to call.
- **Build** (secret, simultaneous, 60 s): put all 4 numbers in any order with **3 of your 4 symbols**
  between them (each symbol once), and declare **Low** (aim for 1) or **High** (aim for 20).
  × and ÷ go before + and −; division may give fractions; dividing by 0 makes the equation invalid.
  Timeout: the best equation for you is built automatically.
- **Betting round 2**, as round 1.
- **Showdown:** equations are shown. The pot is split: half to the Low player closest to 1, half to the
  High player closest to 20 (an odd chip goes to High). If nobody declared a side, the other side takes
  it all. Ties share. If everyone else folded, the last player takes the pot.
- **Score:** your chips at the end. Points: 1000 for first down to 100 for last.
- Moves: `{type:"call"}`, `{type:"raise"}`, `{type:"fold"}`,
  `{type:"build", order:[i,i,i,i], ops:["+","×"…], side:"low"|"high"}` (card indices 0–3; ops as `+ - * /`).
- View: your cards, everyone's face-up cards, chips, the pot, what you must call, your build, the log,
  the last showdown.
- Bot: tries every equation of its own cards and picks the closer target; bets on how close it is.
- Desktop: C calls, R raises, F folds; in the build, 1–4 place cards, + − * / place symbols,
  L / H choose the side, Backspace undoes, Enter submits.
- **Replay moments:** an exact 1 or 20 that wins its side alone (no tie) and takes 18 chips or more
  ("Exactly 20 takes 18 chips: 10 + 10 − 0 ÷ 3", or "Exactly 20 sweeps the pot of 36: …" when nobody
  won the other side), and a pot of 10 or more taken because everyone else folded ("Pot of 22 taken
  without a showdown"). Public: showdown equations are revealed to everyone, and the fold moment shows
  no card.
