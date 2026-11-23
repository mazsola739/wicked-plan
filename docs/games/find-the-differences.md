# Find the Differences

Exile game, a race: 1–4 seats (the page: you + 3 bots). Code: `crates/rules/src/games/find_the_differences.rs`.

## How to play

**Goal:** find every difference between the two pictures before the bots do.

- 3 rounds of 60 seconds. The right picture has 5, 6 and 7 differences in rounds 1–3.
- A difference is a shape in another colour, another kind of shape, another size, or a missing shape.
- Tap the difference in either picture. A find scores +30, but a tap on nothing costs 10, so don't tap wildly.
- Find them all for a race bonus: +100 if you are first, +50 second, +25 third.
- When you have found them all or time runs out, every difference is circled for 4 seconds. Tap **Next pictures** (or **Finish** after the last round) to go on sooner.

## Rules in detail

- Two pictures drawn from the seed: about 14 shapes (circle, square, triangle, star, diamond, hexagon)
  in 8 colours on a coloured ground. The right picture has **5, 6 and 7 differences** in rounds 1–3:
  a shape in another **colour**, another **kind** of shape, another **size**, or **missing**.
- **3 rounds**, **60 seconds** each. Everyone looks at the same pair.
- Tap a difference in **either** picture (within the shape's size + a small margin). A find scores
  **+30**, a tap on nothing scores **−10**.
- **Race bonus** for finding all of them: first **+100**, second **+50**, third **+25** (by time; ties share).
- When you have found them all, or time runs out, the rest of the round is played out at once and every
  difference is circled for **4 seconds** (Next skips). Results: `rank_by_score`.
- Moves: `{type:"tap", x, y}` (picture units 0–1000), `{type:"next"}` (during the reveal).
- View: the round, timer, both pictures (shape lists), how many differences, the ones you found, your
  misses, per rival how many they found, scores; every difference only in the reveal.
- Rivals (every seat after the first) are played by the rules: each has a speed (0.8–1.4) and finds one
  difference every 4–12 s times that speed; each difference has an 8% chance to stay unseen.
- Generator + solver: the generator changes chosen shapes; the solver compares the two pictures and must
  find exactly the generated differences.
- Bot (for the first seat in tests): taps a difference 70% of the time, otherwise a random spot.
- Desktop: N or Enter goes on after the reveal.
