# Heavy 13

Prison game, 3–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/heavy_13.rs`.

- Four coloured **stacks** of rocks (red, blue, green, gold) start empty. A stack holds **13** at most.
- **Cards:** values **1–6** in the four colours, two of each (48 cards). Everyone is dealt **8 cards**,
  and gets **1 rainbow chip** and **2 passes**. **10 rounds** — exactly one card or pass a round.
- **Each round (20 s), everyone secretly at once:**
  - plays a card onto the stack **of its colour**, or
  - plays a card **with the rainbow chip** onto **any** stack (once a game), or
  - **passes** (uses up one pass).
  Timeout: pass if you have a pass left, otherwise your lowest card on its own stack.
- **Reveal:** every card played scores its value for its player at once. Then each stack adds up:
  - **over 13: overloaded.** Everyone who added to it this round **loses twice their card's value**
    (so the card nets minus its value). The stack is emptied.
  - **exactly 13: full.** Everyone who added to it this round gets **+3**. The stack is emptied.
  - otherwise the rocks stay for the next round.
- **Result (5 s)**, then the next round.
- **Ranking:** most points; ties share a rank. Points by rank: 1000 for the first, 100 for the last,
  evenly between.
- **View:** your cards, chip and passes; the four stack totals; everyone's score, cards left, chip,
  passes and whether they have chosen; after each round every play and what happened to each stack.
- **Moves:** `{type:"play", card, stack}` (`card` = colour × 6 + value − 1; `stack` is `null` for the
  card's own colour, or a colour 0–3 to use the rainbow chip), `{type:"pass"}`.
- **Bot:** imagines the others' plays (a pass now and then, otherwise a random card on a random stack)
  and picks the play with the best average outcome; passes and the rainbow chip count as worth a little,
  so it saves them for risky rounds.
- Desktop: 1–8 pick a card, Q W E R choose the stack (the rainbow chip is used when the colour differs),
  Enter plays, P passes.
