// Prints one line per file with its exported names: `npm run map <folder or file>...`. Cheaper than opening the files.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { extname, relative, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');
const CODE = new Set(['.ts', '.tsx', '.js', '.mjs']);
const SKIP = new Set(['node_modules', 'coverage', 'dist', 'build', 'android', '.vite']);
const EXPORT =
  /^export\s+(?:default\s+)?(?:abstract\s+)?(?:async\s+)?(?:function\*?|const|let|class|type|interface|enum)\s+([A-Za-z_$][\w$]*)/gm;
const REEXPORT = /^export\s+(?:type\s+)?\{([^}]*)\}/gm;
const STAR = /^export\s+\*\s+from\s+'([^']+)'/gm;

function files(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(dir, entry.name);
    if (entry.isDirectory()) return SKIP.has(entry.name) || entry.name.startsWith('.') ? [] : files(path);
    return CODE.has(extname(entry.name)) && !/\.(test|spec)\./.test(entry.name) && !entry.name.endsWith('.d.ts') ? [path] : [];
  });
}

function exportsOf(file) {
  const text = readFileSync(file, 'utf8');
  const names = [...text.matchAll(EXPORT)].map((m) => m[1]);
  for (const m of text.matchAll(REEXPORT)) {
    names.push(
      ...m[1]
        .split(',')
        .map((s) =>
          s
            .trim()
            .split(/\s+as\s+/)
            .pop(),
        )
        .filter(Boolean),
    );
  }
  for (const m of text.matchAll(STAR)) names.push(`*${m[1]}`);
  if (/^export\s+default\s+(?!function|class|async|abstract)/m.test(text)) names.push('default');
  return names;
}

const targets = process.argv.slice(2);
if (!targets.length) {
  console.log('Usage: npm run map <folder or file>... (e.g. npm run map packages/rules/src)');
  process.exit(0);
}
for (const target of targets) {
  const path = resolve(ROOT, target);
  for (const file of statSync(path).isDirectory() ? files(path) : [path]) {
    const names = exportsOf(file);
    console.log(`${relative(ROOT, file).replaceAll('\\', '/')}: ${names.length ? names.join(', ') : '-'}`);
  }
}
