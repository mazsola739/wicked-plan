# Magic Square

Hidden game, solo; our take on the exile's "trial" (a 7×7 square in an hour). Code: `crates/rules/src/games/magic_square.rs`.

## How to play

**Goal:** fill the grid so that every row, every column and both diagonals add up to the same number.

- There are three boards: 3×3 in **2 minutes**, 4×4 in **5 minutes** and 5×5 in **10 minutes**.
- Use each number from 1 up to the number of squares exactly once. The target sum is shown: 15, 34 or 65.
- A few numbers are already given and can't be changed.
- Tap a square, then tap a number to write it in (⌫ clears it).
- When the grid is full, tap **Check**. A wrong check costs nothing, so try again.
- Any correct square counts, not only one fixed answer.
- A solved board is worth **150, 300 or 500** points, plus 1 point for every second left on that board.
- **Give up**, or run out of time, and the board scores 0. You move on to the next one.

## Rules in detail

- Three boards: **3×3, 4×4, 5×5**, with **2, 5 and 10 minutes**.
- Fill in 1 … n² once each so that every row, every column and both diagonals add up to the same number
  (15, 34, 65). A few numbers are given and fixed: 1 on the 3×3, 6 on the 4×4, 6 on the 5×5 (always including the 1).
- **Any** correct square counts, not only the one the puzzle was made from.
- Check as often as you like; a wrong check is only counted. Give up or run out of time: 0 points, next board.
- Points: **150, 300, 500** per solved board, plus 1 point for every second left on that board.
- Moves: `{type:"set", cell, value}` (value 0 clears), `{type:"submit"}`, `{type:"skip"}`.
- Bot: copies the solution, with a slip now and then.
- Desktop: type a number (two digits on the bigger boards), arrows move, Backspace clears, Enter checks.
