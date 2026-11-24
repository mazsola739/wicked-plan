# Memory Scene Quiz

Team mini game, 1–4 players (on the page: you + 3 bot teammates). Code:
`crates/rules/src/games/memory_scene_quiz.rs`.

## How to play

**Goal:** remember a scene well enough that your team answers all 10 questions about it correctly.

- The scene has 8 objects in a grid. Each object has a shape, a colour and a count of 1 to 3 copies.
- Study it as long as you like and tap **Ready**. When everyone is ready, it disappears.
- Next come **10 questions** with 4 answers each. They go round the team: you, then each teammate, then you again.
- When it's your question, tap one of the four answers.
- Questions ask about colours, shapes, counts, positions, and which object was not in the scene.
- **One wrong answer ends the game for the whole team.**
- Every right answer earns the team **100 points** (all ten: 1000), and everyone gets the team's score.

## Rules in detail

- **Scene:** a 4 × 3 grid with **8 objects**. Each object has a **shape** (circle, square, triangle, star,
  heart, moon), a **colour** (red, blue, green, yellow, purple) and a **count** (1–3 copies in its cell).
- **Study:** everyone sees the scene until every player taps Ready.
- **10 questions in a row**, multiple choice with 4 answers. The questions go round the team in seat order
  (you, then each teammate, then you again…); only the player whose turn it is answers, everyone sees it.
- **One wrong answer fails the team:** the game ends there.
- Question kinds: the colour of a shape that appears once; the shape in a given cell; how many copies of
  a coloured shape; how many of a shape in all; how many objects of a colour; which object was right of
  another; which of four objects was **not** in the scene.
- **Score (team):** **100 points per right answer** (all ten: 1000). Every player gets the team's points
  and shares rank 1.
- **View:** the scene only while studying (and after the end); the current question, its four answers, who
  answers it, and the questions answered so far. The right answer is never sent before it is answered.
- **Moves:** `{type:"ready"}` while studying, `{type:"answer", choice}` (0–3) by the player whose turn it is.
- **Bot:** taps Ready at once; answers right 95% of the time.
- **Screen:** the scene in SVG; the four answers as big buttons. Desktop keys: 1–4 answer, Enter = Ready.
- **Replay moments:** the tenth question answered right, for a perfect quiz, by whoever answered it
  ("Question 10 right: a perfect 1000"). Public: every answer is shown to the whole team.
