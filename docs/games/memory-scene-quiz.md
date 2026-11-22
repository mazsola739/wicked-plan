# Memory Scene Quiz

Team mini game, 1–4 players (on the page: you + 3 bot teammates). Code:
`crates/rules/src/games/memory_scene_quiz.rs`.

- **Scene:** a 4 × 3 grid with **8 objects**. Each object has a **shape** (circle, square, triangle, star,
  heart, moon), a **colour** (red, blue, green, yellow, purple) and a **count** (1–3 copies in its cell).
- **Study:** everyone sees the scene for **30 seconds**; it ends early when every player taps Ready.
- **10 questions in a row**, multiple choice with 4 answers. The questions go round the team in seat order
  (you, then each teammate, then you again…); only the player whose turn it is answers, everyone sees it.
  **15 seconds** per question.
- **One wrong answer (or running out of time) fails the team:** the game ends there.
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
