/**
 * Rebuild the wasm core when it is missing or older than the Rust sources.
 *
 * A fresh checkout has no `src/wasm`, and both `pnpm dev` and `pnpm build`
 * import it. Rather than failing on a missing module — or quietly running a
 * stale build after a Rust edit — check first and only pay for wasm-pack when
 * it is actually needed.
 */
import { spawnSync } from 'node:child_process';
import { readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';

const OUTPUT = 'src/wasm/btbm_core.js';

/** Newest modification time under `dir`. */
function newestIn(dir) {
    let newest = 0;
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const path = join(dir, entry.name);
        newest = Math.max(newest, entry.isDirectory() ? newestIn(path) : statSync(path).mtimeMs);
    }
    return newest;
}

let built = 0;
try {
    built = statSync(OUTPUT).mtimeMs;
} catch {
    // Never built.
}

const source = Math.max(newestIn('src/rust'), statSync('Cargo.toml').mtimeMs);
if (built >= source) process.exit(0);

console.log(built ? 'Rust sources changed: rebuilding the wasm core.' : 'Building the wasm core.');
// No `shell: true`: it would concatenate the arguments unescaped. Windows is
// the only place the launcher is not simply `pnpm`.
const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
const build = spawnSync(pnpm, ['wasm'], { stdio: 'inherit' });
if (build.status !== 0) {
    console.error('\nThe wasm core did not build. Install the tool with `cargo install wasm-pack`.');
    process.exit(1);
}
