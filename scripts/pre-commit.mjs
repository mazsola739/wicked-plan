// Formats and lints the staged files, then typechecks and tests the workspaces they touch (and the workspaces that import those).
// Run by .githooks/pre-commit; see docs/dev.md.
import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');
const CODE = /\.(ts|tsx|js|mjs|cjs)$/;
const FORMATTABLE = /\.(ts|tsx|js|mjs|cjs|json)$/;
const TAIL_LINES = 40;
// The running bun by its full path: a child cmd.exe cannot start WinGet's bun.exe link by name.
const BUN = `"${process.execPath}"`;
const BIOME = `${BUN} "${join(ROOT, 'node_modules/@biomejs/biome/bin/biome')}"`;

const git = (args) => spawnSync('git', args, { cwd: ROOT, encoding: 'utf8' }).stdout.split('\0').filter(Boolean);
const quote = (files) => files.map((f) => `"${f}"`).join(' ');

function run(label, cmd, cwd = ROOT) {
  const start = Date.now();
  const result = spawnSync(cmd, { cwd, shell: true, encoding: 'utf8', maxBuffer: 1 << 28 });
  const seconds = ((Date.now() - start) / 1000).toFixed(1);
  if (result.status === 0) {
    console.log(`pre-commit: ${label} ok (${seconds}s)`);
    return true;
  }
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`.trimEnd().split('\n');
  console.log(`pre-commit: ${label} FAILED (${seconds}s): ${cmd}`);
  // The failing lines first: tsc and cargo put a summary at the tail.
  const failures = output.filter((line) => /error TS\d+|error\[|FAILED|panicked|✖|×/.test(line));
  if (failures.length) console.log(failures.slice(0, 30).join('\n'), '\n---');
  console.log(output.slice(-TAIL_LINES).join('\n'));
  return false;
}

/** The workspaces (apps/*, packages/*), with the @wicked-plan packages each one imports. */
function workspaces() {
  const globs = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8')).workspaces;
  const out = [];
  for (const glob of globs) {
    const parent = glob.replace(/\/\*$/, '');
    if (!existsSync(join(ROOT, parent))) continue;
    for (const entry of readdirSync(join(ROOT, parent), { withFileTypes: true })) {
      const manifest = join(ROOT, parent, entry.name, 'package.json');
      if (!entry.isDirectory() || !existsSync(manifest)) continue;
      const pkg = JSON.parse(readFileSync(manifest, 'utf8'));
      const deps = Object.keys({ ...pkg.dependencies, ...pkg.devDependencies }).filter((d) => d.startsWith('@wicked-plan/'));
      out.push({ dir: `${parent}/${entry.name}`, name: pkg.name, scripts: pkg.scripts ?? {}, deps });
    }
  }
  return out;
}

const staged = git(['diff', '--cached', '--name-only', '--diff-filter=ACMR', '-z']);
let ok = true;

// Format only fully staged files, so a half-staged file keeps its unstaged lines out of the commit.
const partly = new Set(git(['diff', '--name-only', '-z']));
const toFormat = staged.filter((f) => FORMATTABLE.test(f) && !partly.has(f));
if (toFormat.length) {
  ok = run('format', `${BIOME} format --write --no-errors-on-unmatched ${quote(toFormat)}`) && ok;
  spawnSync('git', ['add', '--', ...toFormat], { cwd: ROOT });
}

const code = staged.filter((f) => CODE.test(f));
if (code.length) ok = run('lint', `${BIOME} lint --no-errors-on-unmatched ${quote(code)}`) && ok;

// A workspace is checked when its own code changed or a workspace it imports changed (transitively).
const all = workspaces();
const changed = new Map(all.map((w) => [w.name, staged.filter((f) => f.startsWith(`${w.dir}/`) && CODE.test(f))]));
const affected = new Set(all.filter((w) => changed.get(w.name).length).map((w) => w.name));
for (let grew = true; grew; ) {
  grew = false;
  for (const w of all) {
    if (!affected.has(w.name) && w.deps.some((d) => affected.has(d))) {
      affected.add(w.name);
      grew = true;
    }
  }
}

for (const w of all) {
  if (!affected.has(w.name)) continue;
  const cwd = join(ROOT, w.dir);
  if (w.scripts.typecheck) ok = run(`${w.dir} typecheck`, `${BUN} run --silent typecheck`, cwd) && ok;
}

// Rust (crates/: rules and the wasm bridge): format check, then every test.
if (staged.some((f) => f.startsWith('crates/') && f.endsWith('.rs'))) {
  const manifest = `--manifest-path "${join(ROOT, 'crates/Cargo.toml')}"`;
  ok = run('rust format', `cargo fmt --all ${manifest} --check`) && ok;
  ok = run('rust tests', `cargo test ${manifest} --quiet`) && ok;
}

process.exit(ok ? 0 : 1);
