# Duplicate Picture Buzzer

Team game, played as a race here: 1–4 seats (the page: you + 3 bots).
Code: `crates/rules/src/games/duplicate_photo_buzzer.rs`.

## How to play

**Goal:** be the first to buzz when a picture repeats.

- There are 6 rounds. In each round, pictures flash one after another.
- Exactly one picture repeats an earlier one from the same round. Tap **BUZZ** while it is on screen or during the picture after it.
- The first right buzz wins the round: 100 points, plus up to 60 more for speed.
- A buzz at any other time costs 30 points, and you are out for the rest of the round.
- **The catch:** some pictures are near copies, with one shape in another colour or kind. They do not count.
- Pictures speed up each round, from 1.4 seconds down to 0.9.
- After each round the repeat and its original are shown; tap **Next round** to go on.

## Rules in detail

- Pictures flash one after another: a coloured ground with three shapes (6 kinds, 8 colours) in one of
  four layouts, all drawn from the seed. No photos.
- **6 rounds.** In each round a stream of 7–14 pictures plays; exactly **one picture repeats** an earlier
  one of the same round. Some pictures are **near copies** (one shape in another colour or kind): they
  do not count.
- A picture stays **1.4 s** in round 1, 0.1 s less each round (0.9 s in round 6).
- **Buzz** while the repeat is on screen or during the picture after it. The first right buzz wins the
  round: **100 + up to 60 for speed** (60 at once, 0 at the end of the two pictures). A buzz at any other
  time is wrong: **−30** and no more buzzing this round.
- The round ends at the first right buzz, or one picture after the repeat. Then the repeat and its
  original are shown for **2.5 s** (Next skips). Results: `rank_by_score`.
- Moves: `{type:"buzz"}`, `{type:"next"}` (during the reveal).
- View: round, the current picture only (never the earlier ones), its number, the picture time, whether you
  are locked out, every seat's points and buzz this round; in the reveal the repeated picture, its two
  numbers and who won.
- Rivals (every seat after the first) are played by the rules: each round each rival reacts
  after the repeat shows (0.7–2.0 s, too slow means no buzz), misses the repeat 30% of the time, and buzzes wrongly
  10% of the time.
- Bot (for the first seat in tests): buzzes on the repeat 80% of the time.
- Desktop: Space or B buzzes, Enter goes on after the reveal.
- **Replay moments:** a right buzz within 0.75 s of the repeat showing ("Repeat buzzed in 0.6 s"),
  or else a round 6 win that takes a seat from behind (or level) to the sole lead ("Lead taken in
  the last round"). Public: the reveal shows who won, and the points are open.
