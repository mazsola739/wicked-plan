# Count the Cubes

Mini solo. Code: `crates/rules/src/games/count_the_cubes.rs`.

## How to play

**Goal:** count the cubes in each stack, fast and right.

- There are 5 rounds of 20 seconds each.
- Each round shows a stack of cubes. Type how many there are on the number pad and tap **OK**.
- A right answer scores 100, plus up to 100 more for speed. A wrong answer or a timeout scores 0.
- The stacks grow bigger each round.
- No cube is ever hidden: a column is never taller than the one behind it, so every cube can be counted from the picture.

## Rules in detail

- **5 rounds**, **20 seconds** each. Each round shows a stack of cubes in an isometric drawing; type how many there are.
- A right answer scores **100 + up to 100 for speed** (`100 × time left / round time`); a wrong answer or a timeout scores 0.
- Boards grow: 3×3 up to 2 high, 3×3 up to 3, 4×3 up to 3, 4×4 up to 4, 5×4 up to 4.
- **Readability:** a column is never taller than the one behind it on either axis, so no cube is hidden and
  the answer can always be counted from the picture.
- Move: `{type:"answer", value}` with 0–999.
- Bot: right 75% of the time, otherwise off by 1–2.
- Desktop: type the number, Enter submits.
