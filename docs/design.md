# Design: Wicked Plan (v4, 2026-11-23)

A **web page of puzzles and party games** in the genre of The Devil's Plan. One page lists every game;
pick one and play. Solo puzzles are played alone; games for several players are played against bots.

The earlier base-building mobile game was dropped on 2026-11-22.

## Decisions so far

- **Platform:** a single-page React + TypeScript web app for desktop (wider layout, keyboard shortcuts)
  and phone (portrait, one hand). No native app.
- **Language:** everything in the product and the docs is English.
- **No rooms and no server for now:** the page is a game picker. A room/server layer may come later;
  the rules keep secrets out of the views so a server could referee without changes.
- **Rules in Rust:** one `crates/rules` crate, run in the browser through WebAssembly.
- **Bots:** in games for several players the other seats are bots that move by themselves
  (seats: the game's minimum, or 4 when it allows more).
- **Original names and look:** the show's games are not copied one to one; we write our own versions.
- **All 51 catalogue games get a digital version** (decided 2026-11-23), built in groups.

## Visual identity (2026-11-23, from the user's key art)

- **Emblem, the Observer:** the user's own artwork (`apps/web/src/assets/observer.png`): a pointed
  hood seen from the front, a black void for a face drawn by glowing LED features (a cyan
  nested-diamond eye, a violet slash eye, a zigzag grin), a W crown above and ring arcs around it.
  The crown alone is the small mark (top bar, footer, favicon), drawn in SVG in `Emblem.tsx`.
- **Palette:** near-black background, dark violet-grey surfaces, **cyan** for highlights and the main
  action, **violet** for fills and the second action, **silver** for the title and line art.
  Difficulty keeps green / yellow / red.
- **Type:** Cinzel (bundled, `@fontsource/cinzel`) for the title and headings; the system sans for
  everything else; tiny labels in letter-spaced capitals.
- **Shapes:** long hexagonal buttons (pointed ends), thin divider lines with a small diamond in the
  middle, thin geometric line art. Glow only on the emblem and focus/hover; `prefers-reduced-motion`
  turns animation off.
- **Inside the games** (decided 2026-11-23): the games share the page's world, not each its own.
  - **Named game colours** are neon on near-black, from one set of tokens: `--g-red`, `--g-orange`,
    `--g-yellow` (also gold), `--g-green`, `--g-cyan`, `--g-blue`, `--g-purple`, `--g-pink`,
    `--g-white`, `--g-black`. A colour the rules name ("red") stays recognisably that colour.
  - **Seats** use one order everywhere: `--seat-0` is you (cyan), then violet, pink, yellow, green,
    orange, blue, white (`--seat-1`…`--seat-7`).
  - **Surfaces:** no felt, wood, soil or sand; boards are `--board` (near-black) with thin silver
    `--board-line` lines. Cards and tiles are dark glass (`--glass`, edge `--glass-edge`) with light
    text and neon symbols, like the floating cards of the key art. Glow is soft and only on what
    matters (your pieces, a selection, a win).
- **Landing:** the emblem and the title on top, a short "How it works", then the catalogue with
  filters (type, difficulty, solo or with bots). No sign-in, lobby or leaderboard until rooms exist.

## Open questions (the user decides)

- Should games keep a score history or best scores on the page?
- Rooms with friends: when, and with which server?

## Milestones

| # | Build | Done when |
|---|---|---|
| P0 | Cleanup: only the puzzles and the tools stay | Tests run, the docs describe the new plan — done |
| P1 | Rules in Rust + WebAssembly; the page with six solo puzzles | Every puzzle can be played through in the browser — done 2026-11-22 |
| P2 | Game picker page, bots for several seats, all 45 remaining games | Every catalogue game can be played through on desktop and phone, its tests (bot games) pass — done 2026-11-23 |
| P3 | Rooms with friends (server) | Two people play one game together |
