<script setup lang="ts" vapor>
/**
 * Milestone editor — developer page.
 *
 * The table in `src/rust/milestones.rs` is the thing being edited, so the page
 * shows it as a flat list ordered by value: term, time, description. The tree
 * underneath is only for *finding* terms that are not milestones yet — it
 * expands a term into its fundamental sequence, the way notation explorer does.
 *
 * Everything is kept in localStorage and pushed to the dev server by **Rebuild**,
 * which rewrites ENTRIES, regenerates the table and rebuilds the wasm in one go.
 */
import { computed, ref } from 'vue';
import { START_MS, familyChain, fsTerm, milestoneRows, ordinalValue } from '../core/wasm';
import { formatMoment } from '../composables/useClock';

interface Node {
    plain: string;
    open: boolean;
    expanded: boolean;
    children: Node[];
    /** Set once the fundamental sequence runs out: nothing left to append. */
    done: boolean;
}

interface Draft {
    title: string;
    desc: string;
}

const DESC_KEY = 'btbm_milestone_drafts_v2';
const MARK_KEY = 'btbm_milestone_marks_v1';

const seeded = milestoneRows();
const seededAt = new Map(seeded.map((r) => [r.ordinal, r]));

/** The limit of the whole notation: a term with no finite expression. */
const LIMIT = 'LIMIT';

function loadDrafts(): Record<string, Draft> {
    const base: Record<string, Draft> = {};
    for (const r of seeded) base[r.ordinal] = { title: r.title, desc: r.desc };
    try {
        const raw = localStorage.getItem(DESC_KEY);
        if (!raw) return base;
        return { ...base, ...(JSON.parse(raw) as Record<string, Draft>) };
    } catch {
        return base;
    }
}

function loadMarks(): Record<string, boolean> {
    const base: Record<string, boolean> = {};
    for (const r of seeded) base[r.ordinal] = true;
    try {
        const raw = localStorage.getItem(MARK_KEY);
        if (!raw) return base;
        return { ...base, ...(JSON.parse(raw) as Record<string, boolean>) };
    } catch {
        return base;
    }
}

const drafts = ref<Record<string, Draft>>(loadDrafts());
const marked = ref<Record<string, boolean>>(loadMarks());
const status = ref('');
const busy = ref(false);
const query = ref('');
const addText = ref('');

function flash(message: string) {
    status.value = message;
    setTimeout(() => {
        if (status.value === message) status.value = '';
    }, 2400);
}

function save() {
    localStorage.setItem(DESC_KEY, JSON.stringify(drafts.value));
    localStorage.setItem(MARK_KEY, JSON.stringify(marked.value));
}

/** Values are fixed per term, so a plain map is enough; the wasm call bisects. */
const values = new Map<string, number | null>();

function valueOf(plain: string): number | null {
    const known = seededAt.get(plain);
    if (known) return known.value;
    if (!values.has(plain)) values.set(plain, ordinalValue(plain));
    return values.get(plain)!;
}

const descOf = (plain: string) => drafts.value[plain]?.desc ?? '';

function setDesc(plain: string, desc: string) {
    if (!drafts.value[plain]) drafts.value[plain] = { title: '', desc: '' };
    drafts.value[plain].desc = desc;
    save();
}

/** Marked terms, ordered by the value at which the walk reaches them. */
const rows = computed(() => {
    const out: { plain: string; value: number | null }[] = [];
    for (const [plain, on] of Object.entries(marked.value)) {
        if (on) out.push({ plain, value: valueOf(plain) });
    }
    return out.sort((a, b) => (a.value ?? Infinity) - (b.value ?? Infinity));
});

const shown = computed(() => {
    const q = query.value.trim().toLowerCase();
    const ranked = rows.value.map((r, i) => ({ ...r, rank: i + 1 }));
    if (!q) return ranked;
    return ranked.filter(
        (r) => r.plain.toLowerCase().includes(q) || descOf(r.plain).toLowerCase().includes(q),
    );
});

const describedCount = computed(
    () => rows.value.filter((r) => descOf(r.plain).trim().length > 0).length,
);

/** Zero is the empty matrix, stored as an empty string. */
const termText = (plain: string) => plain || '0';

const when = (plain: string) => {
    const row = seededAt.get(plain);
    return row ? formatMoment(START_MS + row.atMs) : '—';
};

/** Add a term by hand; it becomes a milestone straight away. */
function addTerm() {
    const plain = addText.value.trim();
    if (!plain) return;
    if (valueOf(plain) === null) {
        flash(`${plain} is not a reachable term`);
        return;
    }
    marked.value[plain] = true;
    if (!drafts.value[plain]) drafts.value[plain] = { title: '', desc: '' };
    addText.value = '';
    save();
    flash(`added ${plain}`);
}

/**
 * Drop a milestone. A described entry has to lose its description first — it
 * was written because somebody decided the term mattered.
 */
function remove(plain: string) {
    if (descOf(plain).trim()) {
        flash('clear the description first');
        return;
    }
    marked.value[plain] = false;
    save();
}

function toggleMark(plain: string) {
    if (marked.value[plain]) {
        remove(plain);
        return;
    }
    marked.value[plain] = true;
    save();
}

/** Every marked term, in value order, as the tuples the table is built from. */
function entries() {
    return rows.value.map((r) => ({
        ordinal: r.plain,
        title: (drafts.value[r.plain]?.title ?? '').trim(),
        desc: descOf(r.plain).trim(),
    }));
}

function entriesText(): string {
    const lit = (s: string) =>
        '"' + s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n') + '"';
    return entries()
        .map((e) => `    (${lit(e.ordinal)}, ${lit(e.title)}, ${lit(e.desc)}),`)
        .join('\n');
}

async function copyEntries() {
    await navigator.clipboard.writeText(entriesText());
    flash(`copied ${rows.value.length} entries`);
}

/**
 * Hand the list to the dev server, which rewrites ENTRIES, regenerates the
 * table and rebuilds the wasm. The page reloads itself once the new wasm is
 * written, so the status below is only visible if that reload is delayed.
 */
async function rebuild() {
    if (busy.value) return;
    busy.value = true;
    status.value = 'rebuilding — this takes a minute…';
    try {
        const res = await fetch('/__dev/rebuild', {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ entries: entries() }),
        });
        const data = (await res.json()) as { ok: boolean; count?: number; log?: string };
        if (data.ok) {
            flash(`rebuilt · ${data.count} entries`);
        } else {
            console.error(data.log);
            flash('rebuild failed — see the console');
        }
    } catch (error) {
        console.error(error);
        flash('rebuild failed');
    } finally {
        busy.value = false;
    }
}

function reset() {
    localStorage.removeItem(DESC_KEY);
    localStorage.removeItem(MARK_KEY);
    drafts.value = loadDrafts();
    marked.value = loadMarks();
    values.clear();
    roots.value = [makeNode(LIMIT)];
    flash('reset');
}

// ---------------------------------------------------------------------------
// Browse tree
// ---------------------------------------------------------------------------

/** How many children one click puts in the tree; `+1` appends more. */
const CHAIN_LENGTH = 9;

function makeNode(plain: string): Node {
    return { plain, open: false, expanded: false, children: [], done: false };
}

const roots = ref<Node[]>([makeNode(LIMIT)]);

/** Bumped after structural changes so `flat` recomputes. */
const tick = ref(0);

const flat = computed(() => {
    tick.value;
    const out: { node: Node; depth: number }[] = [];
    const walk = (nodes: Node[], depth: number) => {
        for (const node of nodes) {
            out.push({ node, depth });
            if (node.open) walk(node.children, depth + 1);
        }
    };
    walk(roots.value, 0);
    return out;
});

/** Clicking a term expands it into its fundamental sequence, like the explorer. */
function onTerm(node: Node) {
    if (!node.expanded) {
        const children: Node[] = [];
        if (node.plain === LIMIT) {
            // The chain of `family(k)` never runs out, so `done` stays false.
            children.push(...familyChain(CHAIN_LENGTH).map(makeNode));
        } else {
            for (let i = 0; i < CHAIN_LENGTH; i++) {
                const term = fsTerm(node.plain, i);
                if (term === null) {
                    node.done = true;
                    break;
                }
                children.push(makeNode(term));
            }
        }
        node.children = children;
        node.expanded = true;
        node.open = children.length > 0;
    } else {
        node.open = !node.open;
    }
    tick.value++;
}

/**
 * Append the next term of the fundamental sequence. `LIMIT` has no finite
 * expression of its own, so its next child is the next `family(k)`.
 */
function more(node: Node) {
    const i = node.children.length;
    const term = node.plain === LIMIT ? (familyChain(i + 1)[i] ?? null) : fsTerm(node.plain, i);
    if (term === null) {
        node.done = true;
    } else {
        node.children.push(makeNode(term));
        node.open = true;
    }
    node.expanded = true;
    tick.value++;
}
</script>

<template>
    <div class="dev">
        <header>
            <h1>Milestones <small>dev</small></h1>
            <a class="back" href="/">← back to the app</a>
        </header>

        <div class="toolbar">
            <span class="count">{{ rows.length }} milestones · {{ describedCount }} described</span>
            <input v-model="query" class="filter" type="search" placeholder="filter" />
            <span class="spacer" />
            <button class="primary" :disabled="busy" @click="rebuild">
                {{ busy ? 'Rebuilding…' : 'Rebuild' }}
            </button>
            <button :disabled="busy" @click="copyEntries">Copy ENTRIES</button>
            <button :disabled="busy" @click="reset">Reset</button>
            <span class="status">{{ status }}</span>
        </div>

        <div class="add">
            <input
                v-model="addText"
                type="text"
                placeholder="paste a term, e.g. (0)(1,1)(2,1)(3,1)(4)"
                @keydown.enter="addTerm"
            />
            <button @click="addTerm">Add milestone</button>
        </div>

        <p class="hint">
            Rows are ordered by value. <b>Rebuild</b> writes <code>ENTRIES</code>, regenerates
            <code>milestones_gen.rs</code> and rebuilds the wasm, then the page reloads.
        </p>

        <ul class="list">
            <li v-for="row in shown" :key="row.plain" class="item">
                <span class="rank">{{ row.rank }}</span>
                <span class="term" :class="{ limit: row.plain === LIMIT }">{{ termText(row.plain) }}</span>
                <span class="when">{{ when(row.plain) }}</span>
                <span class="value">{{
                    row.value === null || !Number.isFinite(row.value) ? '—' : row.value.toFixed(4)
                }}</span>
                <template v-if="row.plain !== LIMIT">
                    <button
                        class="mark on"
                        title="a milestone — clear the description to drop it"
                        @click="remove(row.plain)"
                    >
                        ★
                    </button>
                    <input
                        class="desc"
                        type="text"
                        placeholder="description (optional)"
                        :value="descOf(row.plain)"
                        @input="setDesc(row.plain, ($event.target as HTMLInputElement).value)"
                    />
                </template>
                <span v-else class="desc limit-note">
                    the whole notation's limit — not an ordinal
                </span>
            </li>
        </ul>

        <details class="browse">
            <summary>Browse from the limit</summary>
            <p class="hint">
                Click a term to expand it into its fundamental sequence; click again to fold.
                <b>+1</b> appends the next term, and starring a term adds it to the list above.
            </p>
            <ul class="tree">
                <li v-for="row in flat" :key="row.node.plain + row.depth" class="branch">
                    <div class="row" :style="{ paddingLeft: 6 + row.depth * 18 + 'px' }">
                        <span
                            class="fold"
                            :class="{ empty: !row.node.expanded || row.node.children.length === 0 }"
                            @click.stop="onTerm(row.node)"
                        >
                            {{ row.node.expanded && row.node.children.length ? (row.node.open ? '▼' : '▶') : '·' }}
                        </span>
                        <span
                            class="term"
                            :class="{ limit: row.node.plain === LIMIT }"
                            @click="onTerm(row.node)"
                            >{{ termText(row.node.plain) }}</span
                        >
                        <span class="when">{{ when(row.node.plain) }}</span>
                        <button
                            v-if="row.node.plain !== LIMIT"
                            class="mark"
                            :class="{ on: marked[row.node.plain] }"
                            :title="marked[row.node.plain] ? 'already a milestone' : 'add as a milestone'"
                            @click.stop="toggleMark(row.node.plain)"
                        >
                            {{ marked[row.node.plain] ? '★' : '☆' }}
                        </button>
                        <span v-else class="mark on">★</span>
                        <button
                            v-if="row.node.expanded && !row.node.done"
                            class="more"
                            title="append the next term of the fundamental sequence"
                            @click.stop="more(row.node)"
                        >
                            +1
                        </button>
                    </div>
                </li>
            </ul>
        </details>
    </div>
</template>

<style scoped>
.dev {
    max-width: 1080px;
    margin: 0 auto;
    padding: 0 20px 80px;
}

header {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 0 10px;
    background: linear-gradient(var(--dev-bg) 78%, transparent);
}

h1 {
    font-size: 20px;
    margin: 0 0 4px;
}

h1 small {
    font-size: 11px;
    font-weight: normal;
    color: var(--dev-muted);
    border: 1px solid #ccc;
    border-radius: 8px;
    padding: 0 6px;
}

.back {
    font-size: 12px;
    color: #36c;
}

.toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 6px 0 10px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--dev-line);
}

.spacer {
    flex: 1;
}

.count,
.status {
    font-size: 12px;
    color: var(--dev-muted);
}

.status {
    min-width: 120px;
    text-align: right;
    color: var(--dev-accent);
}

button {
    font: inherit;
    font-size: 12px;
    padding: 3px 10px;
    border: 1px solid var(--dev-line);
    border-radius: 3px;
    background: #fff;
    cursor: pointer;
}

button:hover:not(:disabled) {
    background: var(--dev-accent-soft);
}

button:disabled {
    opacity: 0.5;
    cursor: default;
}

button.primary {
    background: #daf;
    border-color: var(--dev-accent);
}

input {
    font: inherit;
    font-size: 12.5px;
    border: 1px solid var(--dev-line);
    border-radius: 3px;
    padding: 3px 6px;
    background: #fff;
}

input:focus {
    outline: 2px solid var(--dev-accent-soft);
    border-color: var(--dev-accent);
}

.filter {
    width: 160px;
}

.add {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
}

.add input {
    flex: 1;
}

.hint {
    margin: 6px 0 12px;
    font-size: 11.5px;
    color: var(--dev-muted);
    line-height: 1.6;
}

code {
    background: #f2f2f2;
    padding: 0 3px;
    border-radius: 2px;
}

.list,
.tree {
    list-style: none;
    margin: 0;
    padding: 0;
}

.item,
.row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    border-bottom: 1px solid var(--dev-line);
}

.item:hover,
.row:hover {
    background: #f2f6ff;
}

.rank {
    width: 30px;
    text-align: right;
    color: #b3b9c6;
    font-variant-numeric: tabular-nums;
}

.term {
    min-width: 300px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.item .term {
    cursor: default;
}

.row .term:hover {
    text-decoration: underline;
    color: var(--dev-accent);
}

.term.limit {
    color: var(--dev-accent);
    text-transform: uppercase;
    letter-spacing: 0.08em;
}

.when {
    font-size: 11.5px;
    color: var(--dev-muted);
    min-width: 150px;
}

.value {
    font-size: 11.5px;
    color: #b3b9c6;
    min-width: 60px;
    text-align: right;
    font-variant-numeric: tabular-nums;
}

.mark {
    font-size: 13px;
    line-height: 1;
    padding: 2px 4px;
    border: 0;
    background: transparent;
    color: #c3c8d2;
    cursor: pointer;
}

.mark.on {
    color: #e8a300;
}

.mark:hover {
    color: #e8a300;
}

.more {
    font-size: 11px;
    line-height: 1;
    padding: 2px 6px;
    color: var(--dev-accent);
    border-color: transparent;
    background: transparent;
    font-variant-numeric: tabular-nums;
}

.more:hover:not(:disabled) {
    border-color: var(--dev-line);
}

.desc {
    flex: 1;
}

.limit-note {
    border: 0;
    background: transparent;
    color: var(--dev-muted);
    font-style: italic;
    padding-left: 0;
}

.browse {
    margin-top: 22px;
}

.browse summary {
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    color: var(--dev-accent);
    padding: 6px 0;
}

.fold {
    width: 14px;
    color: var(--dev-accent);
    cursor: pointer;
    user-select: none;
    text-align: center;
}

.fold.empty {
    color: #c3c8d2;
    cursor: default;
}
</style>
