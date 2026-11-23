# Time Auction

Prison game, 2–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/time_auction.rs`.

## How to play

**Goal:** win the most tokens by paying with time from your **100-second** bank.

- There are **7 rounds** and one token each round.
- Press and hold the **HOLD** button to join the round. If you are not holding within 15 seconds, you sit it out.
- A **5-second** countdown follows: letting go now means you pass and pay nothing.
- Then the clock runs. Every second you keep holding comes out of your bank. Let go to stop.
- The longest hold wins the token, but everyone pays the time they held. An exact tie wins nothing.
- You never see who else is still holding, and only the winner is announced, not the times.
- Most tokens wins; on equal tokens, more time left wins. (On desktop you can hold Space.)

## Rules in detail

- Everyone starts with a **time bank of 100 seconds** and no tokens. **7 rounds**, one token each.
- **Ready (15 s):** everyone with time left presses and holds the button. The round starts as soon as
  all of them hold it; when the 15 seconds run out, whoever is not holding sits the round out.
- **Countdown (5 s):** letting go now means you pass the round and pay nothing.
- **Bidding:** the clock runs. Every holder pays from their bank for as long as they hold; let go to
  stop. An empty bank lets go by itself. You never see who else is still holding.
- The **longest hold wins the token**. Everyone pays the time they held, winner or not. An exact tie
  for the longest hold gives no token. If nobody bids, nobody wins.
- **Result (4 s):** the winner is announced (not the times); then the next round.
- The game ends after 7 rounds, or earlier when every bank is empty.
- **Ranking:** most tokens; equal tokens: more time left in the bank. Points by rank: 1000 for the
  first, 100 for the last, evenly between.
- **View:** your own bank, hold time and tokens; everyone's tokens; during Ready, who is holding;
  who is out of time; the winner of each past round.
- **Moves:** `{type:"press"}`, `{type:"release"}`, `{type:"wait"}` (does nothing; bots use it to keep holding).
- **Time:** measured in the game's ticks (every 250 ms on the page).
- **Bot:** before each round it picks a plan: sometimes pass in the countdown (more often when its bank is
  low), otherwise hold for about its fair share of the bank (bank ÷ rounds left) times 0.4–2.2,
  a bit longer when it is behind on tokens.
- Desktop: hold Space (or the button).
- **Replay moments:** a third token in a row ("Third token in a row"); a token in round 6 or 7 that
  takes the sole lead in tokens ("Last token of the game takes the sole lead", "Sole lead taken in
  round 6 of 7"); a token won with a hold under 3 s ("Token won with a hold of just 1.2 s"). The first
  two are public (winners and tokens are shown to all); the cheap token only its winner sees, since
  hold times stay secret.
