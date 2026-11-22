# Equation Target

Prison game, played solo here. Code: `crates/rules/src/games/equation_target.rs`.

- **5 rounds**, **40 seconds** each. Ten number cards (1–9) and a target are shown.
- Pick **three different cards** in order and **two operators** (+ − ×). The expression is worked out
  **left to right**: `(a ∘ b) ∘ c`. It must equal the target.
- Targets grow by round: 5–20, 10–30, 15–45, 20–70, 30–99. Every target has at least one solution.
- Scoring: **100 + up to 100 for speed** for a right answer.
- Move: `{type:"answer", value:{picks:[i,j,k], ops:["+","*"]}}` (card indices 0–9).
- Bot: right 70% of the time, otherwise the right cards with random operators.
- Desktop: keys 1–9 and 0 pick the cards in order, + − * set the operators, Backspace undoes, Enter submits.
