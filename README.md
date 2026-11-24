# Wicked Plan

**51 puzzles and party games in your browser**: solo puzzles, and games for several players against
bots. An unofficial fan project, inspired by the Netflix show _The Devil's Plan_.

**Live:** <https://mazsola739.github.io/wicked-plan/> · **Portfolio:** <https://mazsola739.github.io/mazsola739/>

> Fan project. Not affiliated with, endorsed or sponsored by Netflix or the makers of
> _The Devil's Plan_. The games follow the genre and some mechanics of the show's games; their
> names, rules texts, art and look are my own. Classic board games (mill, gomoku, knight's tour)
> keep their names.

## Features

- **51 games** in seven kinds: main games, mini games, team games, prison games, hidden games,
  exile games and final duels. Bluffing, bidding, memory, logic, numbers, words, hidden roles.
- **Bots** take the other seats and play sensibly, not at random.
- **How to play** arrives as a transmission from the Observer, the hooded host who watches you
  play; the big moments of a game come back as a short **replay**.
- **Search and filters** by type, category, difficulty, and solo or with bots.
- **Desktop and phone**: mouse and keyboard shortcuts on a desktop, one-handed touch on a portrait
  phone. With `prefers-reduced-motion` nothing flashes or moves.
- No sign-in, no server, no tracking: everything, fonts and art included, ships with the page.

## How it is built

- **The rules** of every game are a Rust module (`crates/rules/src/games/`) behind one `Game`
  contract: moves, whose turn it is, what each seat may see, the bot, the scores. They are pure and
  seeded: no clock and no other randomness, time arrives as ticks, so the same seed plays the same
  game. Hidden cards, roles and secret numbers stay in the rules' state; the page only ever gets
  the view of your seat, so a server could referee later without changes.
- **Tests**: every game ships with its own tests, and 1,000 random bot games where nothing may
  crash and every game must end. Puzzles get a generator and a solver.
- **The page** is React 19 + TypeScript (`apps/web`); it runs the rules through WebAssembly
  (`crates/wasm`, wasm-bindgen).

## Stack

[Rust](https://www.rust-lang.org) 2024 · WebAssembly ([wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen)) ·
React 19 · TypeScript · [Vite](https://vite.dev) · [Biome](https://biomejs.dev) · [Bun](https://bun.sh)

## Run locally

Once: [Bun](https://bun.sh) 1.4 and [Rust](https://rustup.rs), then

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.106 --locked
bun install
bun run dev
```

| Script              | What it does                                              |
| ------------------- | --------------------------------------------------------- |
| `bun run dev`       | builds the rules to WebAssembly, then serves the page     |
| `bun run wasm`      | rebuilds the rules for the page after a Rust change       |
| `bun run test`      | the Rust tests: rules, bots and the wasm bridge           |
| `bun run typecheck` | TypeScript check of the page                              |
| `bun run lint`      | Biome lint                                                |
| `bun run build`     | production build of the page into `apps/web/dist/`        |

Pushing to `main` runs the checks and deploys to GitHub Pages
(Settings → Pages → Source: "GitHub Actions").

## Docs

[`docs/`](docs/README.md) holds the plan, the game catalogue, the dev tools and one rules page per
game: [`docs/games/`](docs/games/).
