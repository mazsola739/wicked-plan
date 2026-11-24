# Piggy Bank

Prison game, 3–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/piggy_bank.rs`.

## How to play

**Goal:** lose as few points as possible while a piggy bank of coins goes round the table.

- There are 6 rounds. Each round counts one colour, white or black, and starts with an empty bank.
- Holding the bank, add 1 or 2 coins with **+ White** and **+ Black**, then tap **Drop and pass**. Others see how many coins, not their colours.
- When the bank reaches you, you see the coins just dropped. Claim how many coins of the round's colour are inside in total.
- Everyone else taps **Trust** or **Doubt!**.
- If anyone doubts, the bank opens. A right claim costs every doubter a point; a wrong one costs the claimer a point.
- If nobody doubts, the claimer drops coins next. A bank of 10 or more coins opens anyway; a wrong claim costs the claimer a point.
- Tip: an honest claim is the last claim plus what you saw. The fewest points lost wins.

## Rules in detail

- A piggy bank goes round the table. Coins are **white** or **black**; there are always enough.
- **6 rounds.** Each round has a **colour** (white and black take turns, the first one at random) and
  starts with an empty bank.
- **Drop:** the holder secretly drops **1 or 2 coins** of any colours and passes the bank to the
  left. Everyone sees how many coins went in, not their colours.
- **Look and claim:** the receiver peeks through the slot and sees **the coins the previous
  holder just dropped**, then announces how many coins of the round's colour are **inside in total**
  (0 up to the number of coins in the bank).
- **Doubt:** everyone else secretly chooses **doubt** or **trust**.
  - If anyone doubts, the bank is **opened**. A right claim: **every doubter loses a point**. A wrong
    claim: **the claimer loses a point**. The round ends.
  - If nobody doubts, the claim stands. When the bank holds **10 coins or more**, it is opened anyway:
    a wrong claim costs the claimer a point, and the round ends. Otherwise the claimer drops coins and
    passes the bank on.
- **Result (5 s).** The claimer of the opened bank starts the next round.
- Why it works: an honest receiver can add what they saw to the last claim, so a chain of honest claims
  is right. A lie that nobody doubts poisons the next claim, and the liar knows it.
- **Ranking:** everyone starts at 0 and can only lose points; the fewest lost wins, ties share a rank.
  Points by rank: 1000 for the first, 100 for the last, evenly between.
- **View:** the round's colour, coins in the bank, who dropped how many, the claims so far; your own
  drops this round; as the receiver, the coins you saw; after an opening the truth, who doubted and who
  lost a point.
- **Moves:** `{type:"drop", white, black}`, `{type:"claim", count}`, `{type:"doubt"}`, `{type:"trust"}`.
- **Bot:** drops random coins; claims the last claim plus what it saw, but lies by one now and then
  (and then watches the next claim). It doubts when a claim cannot be true given its own coins (or what
  it believed at its own claim), or falls below the last accepted claim, and rarely otherwise.
- Desktop: W / B add a white / black coin, Backspace removes, Enter drops; digits and Enter claim;
  D doubt, T trust.
- **Replay moments:** a right claim that two or more doubters pay for ("Claim of 2 white holds: 2
  doubters pay"), by the claimer; a lone doubter catching a wrong claim in a bank of 6 coins or more
  ("Lone doubt: {bot-2} claimed 4 white, there were 3"), by the doubter. Public: the opened bank, the
  doubters and who lost a point are shown to everyone anyway.
