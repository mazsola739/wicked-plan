# Dice Prediction

Prison game, 2–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/dice_prediction.rs`.

- **6 rounds.** Each round everyone rolls three dice:
  - the **open die**: everyone sees it;
  - the **secret die**: only you see it;
  - the **blind die**: everyone sees it **except you**.
- **Build (15 s):** keep your secret die or **reroll it once** (others see that you rerolled, not the result).
  Timeout: keep.
- **Predict (15 s):** secretly predict **your own rank** at this table this round (1 = best hand).
  Timeout: the middle rank (rounded up).
- **Hands**, best first: **triple** > **straight** (three values in a row, e.g. 3-4-5) > **pair** > nothing.
  Equal kinds compare the sum of the three dice; equal hands share a rank (rank = 1 + the number of
  better hands).
- **Result (6 s):** all dice are shown. Points: **exact prediction +3**, off by one **+1**; the best hand
  of the round (shared on a tie) **+1**.
- **Ranking:** most points; ties share a rank. Points by rank: 1000 for the first, 100 for the last,
  evenly between.
- **View:** your open and secret dice; everyone else's open and blind dice; who rerolled and who is done;
  after each round every die, hand, rank and prediction.
- **Moves:** `{type:"keep"}`, `{type:"reroll"}`, `{type:"predict", rank}`.
- **Bot:** rerolls its secret die when a new roll is worth more on average than the one it has (over
  every value its blind die may have). It predicts the rank that comes up most often when it deals its
  unknown dice (its own blind die, the others' secret dice) at random.
- Desktop: K keep, R reroll, 1–6 predict.
