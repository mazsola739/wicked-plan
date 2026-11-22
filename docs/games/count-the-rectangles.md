# Count the Rectangles

Mini solo. Code: `crates/rules/src/games/count_the_rectangles.rs`.

- **5 rounds**, **30 seconds** each. Each round shows a figure of straight lines; count every rectangle its
  lines draw. Squares count, and so does the outer frame.
- A rectangle counts only when all four sides are drawn along their full length.
- Scoring as in Count the Cubes: **100 + up to 100 for speed**.
- Figures grow: 2×2 cells with 1 extra line, 3×2 with 2, 3×3 with 3, 4×3 with 4, 4×4 with 5. Most extra lines run
  the full width or height; some are shorter pieces.
- Move: `{type:"answer", value}` with 0–999.
- Bot: right 70% of the time.
- Desktop: type the number, Enter submits.
