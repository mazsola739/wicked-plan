# Heavy 13

Prison game, 3–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/heavy_13.rs`.

## How to play

**Goal:** score the most points over 10 rounds by adding rocks to the stacks without overloading them.

- There are four coloured stacks: red, blue, green and gold. Each holds 13 at most.
- You get 8 cards (values 1–6), 1 rainbow chip and 2 passes. Each round everyone plays one card or passes, in secret and at the same time, within 20 seconds.
- Tap a card, then **Play on its colour** or tap its stack. Once a game, the rainbow chip lets you tap any other stack instead. Or tap **Pass**.
- Every card you play scores its value at once.
- If a stack goes over 13, everyone who added to it this round loses twice their card's value, and the stack is emptied.
- If a stack hits exactly 13, everyone who added to it gets +3, and it is emptied.
- If time runs out, you pass if you can, otherwise your lowest card goes on its own colour.

## Rules in detail

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
- **Replay moments:** a card played with the rainbow chip onto a stack that ends the round at exactly 13
  ("Rainbow 4 fills gold to exactly 13"), and a player who was behind before the last round and ends
  it alone in the lead ("Lead stolen in the very last round"). Public: every play is shown after the round.
- Desktop: 1–8 pick a card, Q W E R choose the stack (the rainbow chip is used when the colour differs),
  Enter plays, P passes.
