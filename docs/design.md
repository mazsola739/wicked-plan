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
  From the user's poster (2026-11-23): `hood.webp`, the hooded Observer large and without crown or
  rings (the landing page), and `face.webp`, its LED eyes and grin alone on transparent (the game bar).
- **Palette:** near-black background, dark violet-grey surfaces, **cyan** for highlights and the main
  action, **violet** for fills and the second action, **silver** for the title and line art.
  Difficulty keeps green / yellow / red.
- **Type:** Cinzel (bundled, `@fontsource/cinzel`) for the title and headings; the system sans for
  everything else; tiny labels in letter-spaced capitals.
- **Shapes:** long hexagonal buttons (pointed ends); filter chips, step badges and difficulty pills
  are small hexagons too; thin divider lines with a small diamond in the middle, thin geometric line art. Glow only on the emblem and focus/hover; `prefers-reduced-motion`
  turns animation off.
- **Inside the games** (decided 2026-11-23): the games share the page's world, not each its own.
  - **Named game colours** are neon on near-black, from one set of tokens: `--g-red`, `--g-orange`,
    `--g-yellow` (also gold), `--g-green`, `--g-cyan`, `--g-blue`, `--g-purple`, `--g-pink`,
    `--g-white`, `--g-black`. A colour the rules name ("red") stays recognisably that colour, and a
    thing that *is* a colour (a red card, a yellow stack, a green square) is **filled** with it
    (softened a little toward the dark, e.g. `color-mix(in srgb, var(--g-red) 80%, #15121f)`), not
    dark glass with a coloured edge. Dark glass is for neutral things: number cards, playing cards.
  - **Seats** use one order everywhere: `--seat-0` is you (cyan), then violet, pink, yellow, green,
    orange, blue, white (`--seat-1`…`--seat-7`).
  - **Surfaces:** no felt, wood, soil or sand; boards are `--board` (near-black) with thin silver
    `--board-line` lines. Cards and tiles are dark glass (`--glass`, edge `--glass-edge`) with light
    text and neon symbols, like the floating cards of the key art. Glow is soft and only on what
    matters (your pieces, a selection, a win).
- **Broadcast feel** (decided 2026-11-23): the games feel like a televised mind-game show where an
  unseen host watches and speaks to you over a live feed. Our own wording and look, no lines or
  images from films.
  - Opening a game switches it on like an old monitor (a short CRT flash, never blocking).
  - How to play is a **transmission from the Observer**: the emblem on a scan-lined "monitor", a red
    LIVE dot (it blinks a few times, then stays) and a channel tag (CH and the game's catalogue
    number; nothing counts up, the game waits). The Goal line types itself out, the bullets follow;
    a tap or any key shows everything at once.
  - **The Observer watches**: on the landing page the hooded Observer's signal glitches every 7–16 s
    (one soft pulse and a cyan/violet split, under three flashes per second, WCAG 2.3.1). During a
    game only its **face** is in the bar (`face.webp`), without the hood, and mostly not at all:
    every 9–20 s its piercing gaze shines through the dark: the face surfaces blurred and dim,
    sharpens and glows, flickers like a faulty light, glances aside and fades away, as if peeking
    at you.
  - **Replays** (decided 2026-11-23): when anyone (you or a bot) makes a big move, the game freezes
    on that moment for about 3 s under a TV-style REPLAY banner: whose move it was and one line
    on what happened. The game clock stands still meanwhile (no ticks, bots wait); a tap or any key
    skips it. Each game names its own big moments in its rules (a capture, a sniped hand, a pot won
    with the lowest card, a perfect bid, a winning line...), rare enough to stay special, and its
    rules page lists them under "Replay moments". A replay never shows what that seat could not see.
  - With `prefers-reduced-motion` none of this moves: no flash, no typing, no glitches.
- **Landing:** the hooded Observer and the title on top, a short "How it works", then the catalogue
  with a search field and filters (type, category, difficulty, solo or with bots). No sign-in, lobby or leaderboard until rooms exist.

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
