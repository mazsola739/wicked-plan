# Dev tools

## Commands (from the repo root)

| Command | What it does |
|---|---|
| `npm run dev` | builds the rules to WebAssembly, then serves the page on port 5173; on a phone open the "Network" address (same Wi-Fi) |
| `npm run wasm` | rebuilds `apps/web/src/wasm/` after a Rust change |
| `npm test` | `cargo test` for the rules and the wasm bridge |
| `npm run typecheck` | TypeScript check of the page |
| `npm run lint` | Biome lint |
| `npm run format` | Biome format of the whole repo (rarely needed; the pre-commit formats changed files) |
| `npm run build` | production build of the page |
| `npm run tunnel` | public HTTPS address for the dev server (see below) |
| `npm run map <folder>` | one line per file with its exports; cheaper than opening the files |
| `npm run hooks` | turns on the pre-commit check (once per clone) |

One-time setup for the Rust side: `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli --version 0.2.106 --locked` (the version must match the crate).

## Phone test through a tunnel

When the phone is not on the same Wi-Fi, or you need HTTPS (for example to send the link to a friend):

1. Install `cloudflared` once (Windows: `winget install Cloudflare.cloudflared`). It is already on this machine.
2. Two terminals in the repo root: `npm run dev` and `npm run tunnel`.
3. `npm run tunnel` prints an `https://something.trycloudflare.com` address: open it on the phone.

- The address changes on every start and lives only while both commands run.
- `allowedHosts` in `apps/web/vite.config.ts` lets `*.trycloudflare.com` through.

## Pre-commit check

After `npm run hooks`, `scripts/pre-commit.mjs` runs before every commit:

1. formats the fully staged code files (`.ts`, `.tsx`, `.mjs`, `.json` …) with Biome and stages them again;
2. lints the changed code with Biome;
3. typechecks every npm workspace with a changed file;
4. when a Rust file under `crates/` changed: `cargo fmt --check` and `cargo test`.

A failure stops the commit. Never skip it with `--no-verify`: fix the problem.
Run it by hand with `npm run precommit`.

## Style

- TypeScript: `biome.json` — 2 spaces, single quotes, semicolons, trailing commas, 140-character lines.
- Rust: `crates/rustfmt.toml` — 140-character lines; `cargo clippy --all-targets` should stay clean.

## Claude Code helpers

- **Session-start map:** the SessionStart hook in `.claude/settings.json` runs `scripts/project-context.mjs`:
  workspaces, commands, the `docs/README.md` table, the sections of the big pages, git state.
- **Agents** (`.claude/agents/`, Haiku, cheap): `scout` searches code and docs read-only and answers with
  `path:line` references; `validator` runs typecheck, lint and tests and reports only the failures.
- **Preview:** `.claude/launch.json` has `web` (the page).
