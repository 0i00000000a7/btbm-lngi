/**
 * Dev-only endpoint behind the milestone page's Rebuild button.
 *
 * Editing a description used to mean: copy ENTRIES, paste into
 * `milestones.rs`, run the generator, rebuild the wasm, reload the page. This
 * does all four: rewrite ENTRIES, run `cargo run --example gen_milestones`,
 * then `wasm-pack`. Vite reloads the page once the new wasm lands.
 *
 * It writes to the source tree, so it is registered for `vite serve` only.
 */
import { execFile } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { promisify } from 'node:util';
import type { Plugin } from 'vite';

const run = promisify(execFile);

interface Entry {
    ordinal: string;
    title: string;
    desc: string;
}

interface Reading {
    label: string;
    latex: boolean;
    term: string;
}

const HEADER = 'pub const ENTRIES: &[(&str, &str, &str)] = &[';
const READINGS_HEADER = 'pub const EXTRA_READINGS: &[(&str, &[Reading])] = &[';

/** A Rust string literal; the table is ASCII plus a few accented names. */
function literal(s: string): string {
    return '"' + s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\r?\n/g, '\\n') + '"';
}

/** Replace the ENTRIES block in place, leaving the rest of the file alone. */
async function writeEntries(root: string, entries: Entry[]): Promise<number> {
    const file = path.join(root, 'src/rust/milestones.rs');
    const source = await readFile(file, 'utf8');

    const start = source.indexOf(HEADER);
    if (start < 0) throw new Error(`no ENTRIES block in ${file}`);
    const bodyStart = start + HEADER.length;
    const end = source.indexOf('\n];', bodyStart);
    if (end < 0) throw new Error('ENTRIES block is not terminated');

    const body =
        '    // Written by the developer page (dev.html); edit there and press Rebuild.\n' +
        entries
            .map((e) => `    (${literal(e.ordinal)}, ${literal(e.title)}, ${literal(e.desc)}),`)
            .join('\n');

    await writeFile(file, source.slice(0, bodyStart) + '\n' + body + source.slice(end), 'utf8');
    return entries.length;
}

/**
 * Replace the EXTRA_READINGS block: hand-written readings for the landmarks a
 * converter cannot reach, keyed by the spelling the generated table stores.
 *
 * They are merged into the table by the generator, so the app pays nothing at
 * runtime — the same reason the automatic readings are baked.
 */
async function writeReadings(
    root: string,
    readings: Record<string, Reading[]>,
): Promise<number> {
    const file = path.join(root, 'src/rust/milestones.rs');
    const source = await readFile(file, 'utf8');

    const start = source.indexOf(READINGS_HEADER);
    if (start < 0) throw new Error('no EXTRA_READINGS block in ' + file);
    const bodyStart = start + READINGS_HEADER.length;
    const end = source.indexOf('\n];', bodyStart);
    if (end < 0) throw new Error('EXTRA_READINGS block is not terminated');

    const rows: string[] = [];
    for (const [ordinal, list] of Object.entries(readings)) {
        // A row with a label but no term is a request to drop that reading.
        const kept = list.filter((row) => row.label.trim());
        if (kept.length === 0) continue;
        const items = kept
            .map(
                (row) =>
                    '        Reading { label: ' +
                    literal(row.label.trim()) +
                    ', latex: ' +
                    row.latex +
                    ', term: ' +
                    literal(row.term.trim()) +
                    ' },',
            )
            .join('\n');
        rows.push('    (' + literal(ordinal) + ', &[\n' + items + '\n    ]),');
    }

    const body = rows.length
        ? '    // Written by the developer page (dev.html); edit there and press Rebuild.\n' +
          rows.join('\n')
        : '';
    await writeFile(file, source.slice(0, bodyStart) + '\n' + body + source.slice(end), 'utf8');
    return rows.length;
}

const exec = { maxBuffer: 32 << 20, timeout: 30 * 60_000 };

export function milestoneRebuild(): Plugin {
    return {
        name: 'btbm:milestone-rebuild',
        apply: 'serve',
        configureServer(server) {
            server.middlewares.use('/__dev/rebuild', (req, res) => {
                const reply = (code: number, payload: unknown) => {
                    res.statusCode = code;
                    res.setHeader('content-type', 'application/json');
                    res.end(JSON.stringify(payload));
                };

                if (req.method !== 'POST') {
                    reply(405, { ok: false, log: 'POST only' });
                    return;
                }

                let raw = '';
                req.on('data', (chunk) => (raw += chunk));
                req.on('end', async () => {
                    const log: string[] = [];
                    try {
                        const { entries, readings } = JSON.parse(raw) as {
                            entries: Entry[];
                            readings?: Record<string, Reading[]>;
                        };
                        const root = server.config.root;
                        const supplemented = await writeReadings(root, readings ?? {});
                        log.push('wrote ' + supplemented + ' supplemented landmarks');
                        log.push(`wrote ${await writeEntries(root, entries)} entries`);

                        const gen = await run(
                            'cargo',
                            ['run', '--release', '--example', 'gen_milestones'],
                            { cwd: root, ...exec },
                        );
                        log.push(gen.stdout.trim());

                        const wasm = await run('pnpm', ['wasm'], { cwd: root, ...exec });
                        log.push(wasm.stdout.trim());

                        reply(200, { ok: true, count: entries.length, log: log.join('\n') });
                    } catch (err) {
                        const e = err as { message?: string; stdout?: string; stderr?: string };
                        log.push(e.message ?? String(err), e.stdout ?? '', e.stderr ?? '');
                        reply(500, { ok: false, log: log.join('\n') });
                    }
                });
            });
        },
    };
}
