<script setup lang="ts" vapor>
/**
 * Milestone editor — developer page.
 *
 * The tree is the whole page, in the style of Notation Explorer rewritten: an
 * outline whose rows read strictly decreasing from top to bottom. It starts at
 * the synthetic LIMIT, lists the family chain largest first, and a click grows
 * a term into its fundamental sequence.
 *
 * A term is only inserted between a node and the row below it when it is
 * smaller than the node and larger than that row, which is what keeps the
 * outline decreasing. The star marks a term as a milestone and opens its
 * description, which is what Rebuild writes back into ENTRIES.
 */
import { computed, ref } from 'vue';
import {
    START_MS,
    familyChain,
    fsTerm,
    milestoneRows,
    ordinalValue,
    renderOrdinal,
} from '../core/wasm';
import { formatMoment } from '../composables/useClock';
import { LIMIT, compareOrdinal } from './compare';

interface Draft {
    title: string;
    desc: string;
}

/**
 * A hand-written reading: the landmarks past every converter's ceiling get
 * theirs from here. Rebuild writes them into EXTRA_READINGS, which the
 * generator folds into the table.
 */
interface Reading {
    label: string;
    /** Whether the term is LaTeX, typeset by KaTeX, or plain text. */
    latex: boolean;
    term: string;
}

interface Node {
    plain: string;
    children: Node[];
    parent: Node | null;
    /** Stable key; assigned on creation and never reused. */
    id: number;
    /** Next fundamental-sequence index to try. */
    nextIndex: number;
    expanded: boolean;
    open: boolean;
    done: boolean;
}

const DESC_KEY = 'btbm_milestone_drafts_v2';
const MARK_KEY = 'btbm_milestone_marks_v1';
const READINGS_KEY = 'btbm_milestone_readings_v1';

/** Terms added per click. */
const STEP = 1;
/** Family terms the top level starts with, so there is somewhere to begin. */
const INITIAL = 9;
/** Give up looking for a term that fits after this many attempts. */
const MAX_SCAN = 96;
/** How far the family chain is precomputed. */
const FAMILY_COUNT = 32;

const families = familyChain(FAMILY_COUNT);

const seeded = milestoneRows();
const seededAt = new Map(seeded.map((r) => [r.ordinal, r]));

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

function loadReadings(): Record<string, Reading[]> {
    try {
        const raw = localStorage.getItem(READINGS_KEY);
        return raw ? (JSON.parse(raw) as Record<string, Reading[]>) : {};
    } catch {
        return {};
    }
}

const readings = ref<Record<string, Reading[]>>(loadReadings());
const drafts = ref<Record<string, Draft>>(loadDrafts());
const marked = ref<Record<string, boolean>>(loadMarks());
const status = ref('');
const busy = ref(false);
const addText = ref('');
/** The last term added by hand, so its description can be edited. */
const added = ref('');

function flash(message: string) {
    status.value = message;
    setTimeout(() => {
        if (status.value === message) status.value = '';
    }, 2400);
}

function save() {
    localStorage.setItem(DESC_KEY, JSON.stringify(drafts.value));
    localStorage.setItem(MARK_KEY, JSON.stringify(marked.value));
    localStorage.setItem(READINGS_KEY, JSON.stringify(readings.value));
}

const descOf = (plain: string) => drafts.value[plain]?.desc ?? '';

function setDesc(plain: string, desc: string) {
    if (!drafts.value[plain]) drafts.value[plain] = { title: '', desc: '' };
    drafts.value[plain].desc = desc;
    save();
}

/** The readings the table carries that this page has not touched. */
function bakedReadings(plain: string): Reading[] {
    const own = readings.value[plain] ?? [];
    return (seededAt.get(plain)?.readings ?? []).filter(
        (row) => !own.some((mine) => mine.label === row.label),
    );
}

/**
 * The page's own rows, which are the editable ones.
 *
 * A row whose label the table already has and whose term is empty is a
 * deletion marker, not something to show: the reading was dropped.
 */
const readingsOf = (plain: string): Reading[] =>
    (readings.value[plain] ?? []).filter(
        (row) =>
            row.term.trim().length > 0 ||
            !(seededAt.get(plain)?.readings ?? []).some((baked) => baked.label === row.label),
    );

/** Copy a table reading into the editor, so it can be corrected. */
function overrideReading(plain: string, label: string) {
    const baked = seededAt.get(plain)?.readings.find((row) => row.label === label);
    if (!baked) return;
    const list = (readings.value[plain] ??= []);
    if (!list.some((row) => row.label === label)) list.push({ ...baked });
    save();
}

/** Drop a table reading: an empty term on a known label means remove it. */
function suppressReading(plain: string, label: string) {
    const list = (readings.value[plain] ??= []);
    const at = list.findIndex((row) => row.label === label);
    if (at >= 0) list.splice(at, 1);
    list.push({ label, latex: false, term: '' });
    save();
}

function addReading(plain: string) {
    // Sequences are the common case, so a new row starts as plain text.
    (readings.value[plain] ??= []).push({ label: '', latex: false, term: '' });
    save();
}

function removeReading(plain: string, row: Reading) {
    const list = readings.value[plain];
    if (!list) return;
    const at = list.indexOf(row);
    if (at >= 0) list.splice(at, 1);
    if (list.length === 0) delete readings.value[plain];
    save();
}

function setReading(row: Reading, field: 'label' | 'term', value: string) {
    row[field] = value;
    save();
}

function toggleLatex(row: Reading) {
    row.latex = !row.latex;
    save();
}

/** How many landmarks carry a hand-written reading. */
const supplemented = computed(
    () => Object.values(readings.value).filter((list) => list.length > 0).length,
);

/** Values are fixed per term, so a plain map is enough; the wasm call bisects. */
const values = new Map<string, number | null>();

function valueOf(plain: string): number | null {
    const known = seededAt.get(plain);
    if (known) return known.value;
    if (!values.has(plain)) values.set(plain, ordinalValue(plain));
    return values.get(plain)!;
}

/** Marked terms, ordered by the value at which the walk reaches them. */
const rows = computed(() => {
    const out: { plain: string; value: number | null }[] = [];
    for (const [plain, on] of Object.entries(marked.value)) {
        if (on) out.push({ plain, value: valueOf(plain) });
    }
    return out.sort((a, b) => (a.value ?? Infinity) - (b.value ?? Infinity));
});

const describedCount = computed(
    () => rows.value.filter((r) => descOf(r.plain).trim().length > 0).length,
);

/** Zero is the empty matrix, stored as an empty string. */
const termText = (plain: string) => plain || '0';

/** The three structures the tree can be read in. */
const MODES = [
    { id: 'layer', label: 'Layer' },
    { id: 'index', label: 'Index' },
    { id: 'annotated', label: 'Annotated' },
] as const;

type Mode = (typeof MODES)[number]['id'];

const MODE_KEY = 'btbm_milestone_mode_v1';

function loadMode(): Mode {
    const raw = localStorage.getItem(MODE_KEY);
    return raw === 'index' || raw === 'annotated' ? raw : 'layer';
}

const mode = ref<Mode>(loadMode());

const htmlCache = new Map<string, string>();

const escapeHtml = (text: string) =>
    text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

/**
 * The term as markup in the chosen structure.
 *
 * The walk's plain spelling stays the identity of a row; only the drawing
 * changes, so the tree's order is the same in every mode.
 */
function termHtml(plain: string): string {
    if (!plain || plain === LIMIT) return escapeHtml(termText(plain));
    const key = mode.value + '|' + plain;
    const hit = htmlCache.get(key);
    if (hit !== undefined) return hit;
    let html: string;
    try {
        html = renderOrdinal(plain, mode.value, false, 0);
    } catch {
        html = escapeHtml(plain);
    }
    if (htmlCache.size > 4096) htmlCache.clear();
    htmlCache.set(key, html);
    return html;
}

function setMode(id: Mode) {
    mode.value = id;
    localStorage.setItem(MODE_KEY, id);
}

const when = (plain: string) => {
    const row = seededAt.get(plain);
    return row && Number.isFinite(row.atMs) ? formatMoment(START_MS + row.atMs) : '';
};

// ---------------------------------------------------------------------------
// The tree
// ---------------------------------------------------------------------------

let nextId = 1;

function makeNode(plain: string, parent: Node | null): Node {
    return {
        plain,
        children: [],
        parent,
        id: nextId++,
        nextIndex: 0,
        expanded: false,
        open: false,
        done: false,
    };
}

/** The next node in pre-order: the first child, else the next sibling. */
function successor(node: Node): Node | null {
    if (node.children.length > 0) return node.children[0];
    let cur: Node | null = node;
    while (cur && cur.parent) {
        const siblings = cur.parent.children;
        const i = siblings.indexOf(cur);
        if (i < siblings.length - 1) return siblings[i + 1];
        cur = cur.parent;
    }
    return null;
}

/** The i-th fundamental-sequence term of the node, or null once exhausted. */
function nextTerm(node: Node, i: number): string | null {
    if (node.plain === LIMIT) return families[i] ?? null;
    return fsTerm(node.plain, i);
}

/**
 * Add up to count terms to the node, each strictly between the node and the
 * row below it.
 *
 * Terms increase along the fundamental sequence, so the batch is reversed
 * before it is prepended: the largest lands directly under the node, which is
 * what keeps the outline strictly decreasing.
 */
function grow(node: Node, count: number): number {
    const bound = successor(node)?.plain;
    const accepted: string[] = [];
    let scans = 0;
    while (accepted.length < count && scans < MAX_SCAN && !node.done) {
        const term = nextTerm(node, node.nextIndex);
        node.nextIndex++;
        scans++;
        if (term === null) {
            node.done = true;
            break;
        }
        // Not below the node, not above the row underneath, not a repeat.
        if (compareOrdinal(term, node.plain) >= 0) continue;
        if (bound !== undefined && compareOrdinal(term, bound) <= 0) continue;
        const previous = accepted[accepted.length - 1];
        if (previous !== undefined && compareOrdinal(term, previous) <= 0) continue;
        accepted.push(term);
    }
    node.children.unshift(...accepted.reverse().map((plain) => makeNode(plain, node)));
    return accepted.length;
}

const roots = ref<Node[]>([]);
/** Bumped after every structural change so the row list recomputes. */
const tick = ref(0);

function addBatch(node: Node) {
    const grown = grow(node, STEP);
    node.expanded = true;
    if (grown > 0) node.open = true;
    tick.value++;
    if (grown === 0) flash('no more terms here');
}

/** Clicking a term grows it; clicking a folded one opens it again. */
function onTerm(node: Node) {
    if (!node.expanded) {
        addBatch(node);
        return;
    }
    if (!node.open) {
        node.open = true;
        tick.value++;
        return;
    }
    addBatch(node);
}

/** The arrow only folds and unfolds. */
function onFold(node: Node) {
    if (!node.expanded) {
        addBatch(node);
        return;
    }
    node.open = !node.open;
    tick.value++;
}

function initTree() {
    const root = makeNode(LIMIT, null);
    grow(root, INITIAL);
    root.expanded = true;
    root.open = true;
    roots.value = [root];
    tick.value++;
}

/** Fold everything, leaving just the limit. */
function collapseAll() {
    const walk = (nodes: Node[]) => {
        for (const node of nodes) {
            node.open = false;
            walk(node.children);
        }
    };
    walk(roots.value);
    tick.value++;
    flash('folded');
}

interface Row {
    node: Node;
    depth: number;
    /** One class list per indent column, left to right. */
    guides: string[];
    last: boolean;
}

const flat = computed<Row[]>(() => {
    tick.value;
    const out: Row[] = [];
    const walk = (nodes: Node[], depth: number, path: boolean[]) => {
        for (let i = 0; i < nodes.length; i++) {
            const node = nodes[i];
            const last = i === nodes.length - 1;
            const rowPath = [...path, last];
            const guides: string[] = [];
            for (let k = 0; k < depth; k++) {
                const isParentColumn = k === depth - 1;
                const isLast = rowPath[k + 1];
                if (isParentColumn) guides.push(isLast ? 'elbow stop' : 'elbow');
                else guides.push(isLast ? '' : 'vline');
            }
            out.push({ node, depth, guides, last });
            if (node.open && node.children.length) walk(node.children, depth + 1, rowPath);
        }
    };
    walk(roots.value, 0, []);
    return out;
});

function mark(plain: string, on: boolean) {
    marked.value[plain] = on;
    if (on && !drafts.value[plain]) drafts.value[plain] = { title: '', desc: '' };
    save();
}

function toggleMark(plain: string) {
    mark(plain, !marked.value[plain]);
}

function addTerm() {
    const plain = addText.value.trim();
    if (!plain) return;
    mark(plain, true);
    added.value = plain;
    addText.value = '';
    flash('added ' + plain);
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
        .map((e) => '    (' + lit(e.ordinal) + ', ' + lit(e.title) + ', ' + lit(e.desc) + '),')
        .join('\n');
}

async function copyEntries() {
    await navigator.clipboard.writeText(entriesText());
    flash('copied ' + rows.value.length + ' entries');
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
            body: JSON.stringify({ entries: entries(), readings: readings.value }),
        });
        const data = (await res.json()) as { ok: boolean; count?: number; log?: string };
        if (data.ok) {
            flash('rebuilt · ' + data.count + ' entries');
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
    localStorage.removeItem(READINGS_KEY);
    drafts.value = loadDrafts();
    marked.value = loadMarks();
    readings.value = {};
    values.clear();
    added.value = '';
    initTree();
    flash('reset');
}

initTree();
</script>

<template>
    <div class="dev">
        <header class="head">
            <div class="bar">
                <span class="title">Milestones <em>dev</em></span>
                <span class="count"
                    >{{ rows.length }} marked · {{ describedCount }} described ·
                    {{ supplemented }} supplemented</span
                >
                <div class="modes">
                    <button
                        v-for="m in MODES"
                        :key="m.id"
                        :class="{ active: mode === m.id }"
                        @click="setMode(m.id)"
                    >
                        {{ m.label }}
                    </button>
                </div>
                <span class="spacer" />
                <button :disabled="busy" @click="collapseAll">Fold</button>
                <button :disabled="busy" @click="copyEntries">Copy</button>
                <button class="primary" :disabled="busy" @click="rebuild">
                    {{ busy ? 'Rebuilding…' : 'Rebuild' }}
                </button>
                <button class="danger" :disabled="busy" @click="reset">Reset</button>
            </div>
            <div class="bar">
                <input
                    v-model="addText"
                    class="add-input"
                    type="text"
                    placeholder="add a term by hand, e.g. (0)(1,1)"
                    @keydown.enter="addTerm"
                />
                <button @click="addTerm">Add</button>
                <span class="status">{{ status }}</span>
            </div>
        </header>

        <div v-if="added" class="added">
            <span class="added-label">added</span>
            <span class="term" v-html="termHtml(added)"></span>
            <input
                class="desc"
                type="text"
                placeholder="description"
                :value="descOf(added)"
                @input="setDesc(added, ($event.target as HTMLInputElement).value)"
            />
            <button class="close" @click="added = ''">×</button>
        </div>

        <p class="hint">
            Click a term to add the next term of its fundamental sequence; the arrow folds it. The
            star marks a milestone and opens its description.
        </p>

        <div class="tree">
            <div v-for="row in flat" :key="row.node.id" class="row">
                <span v-for="(cls, k) in row.guides" :key="k" class="guide" :class="cls"></span>
                <div class="body" :class="{ marked: marked[row.node.plain] }">
                    <div class="line">
                        <span
                            class="fold"
                            :class="{ leaf: !row.node.expanded }"
                            @click="onFold(row.node)"
                            >{{ row.node.expanded ? (row.node.open ? '▾' : '▸') : '▸' }}</span
                        >
                        <span
                            class="term"
                            :class="{
                                limit: row.node.plain === LIMIT,
                                grown: row.node.expanded && row.node.children.length > 0,
                            }"
                            @click="onTerm(row.node)"
                            v-html="termHtml(row.node.plain)"
                        ></span>
                        <span v-if="when(row.node.plain)" class="when">{{
                            when(row.node.plain)
                        }}</span>
                        <button
                            class="star"
                            :class="{ on: marked[row.node.plain] }"
                            :title="marked[row.node.plain] ? 'milestone' : 'mark as a milestone'"
                            @click.stop="toggleMark(row.node.plain)"
                        >
                            {{ marked[row.node.plain] ? '★' : '☆' }}
                        </button>
                    </div>
                    <input
                        v-if="marked[row.node.plain]"
                        class="desc"
                        type="text"
                        placeholder="description"
                        :value="descOf(row.node.plain)"
                        @input="setDesc(row.node.plain, ($event.target as HTMLInputElement).value)"
                    />
                    <div v-if="marked[row.node.plain]" class="readings">
                        <span
                            v-for="baked in bakedReadings(row.node.plain)"
                            :key="'baked-' + baked.label"
                            class="baked"
                        >
                            <span class="notation">{{ baked.label }}</span>{{ baked.term }}
                            <button
                                class="mini"
                                title="edit this reading"
                                @click="overrideReading(row.node.plain, baked.label)"
                            >
                                ✎
                            </button>
                            <button
                                class="mini"
                                title="drop this reading"
                                @click="suppressReading(row.node.plain, baked.label)"
                            >
                                ×
                            </button>
                        </span>
                        <div
                            v-for="(reading, i) in readingsOf(row.node.plain)"
                            :key="'own-' + i"
                            class="read-edit"
                        >
                            <input
                                class="read-label"
                                type="text"
                                placeholder="notation"
                                :value="reading.label"
                                @input="
                                    setReading(
                                        reading,
                                        'label',
                                        ($event.target as HTMLInputElement).value,
                                    )
                                "
                            />
                            <input
                                class="read-term"
                                type="text"
                                placeholder="term"
                                :value="reading.term"
                                @input="
                                    setReading(
                                        reading,
                                        'term',
                                        ($event.target as HTMLInputElement).value,
                                    )
                                "
                            />
                            <button
                                class="read-tex"
                                :class="{ on: reading.latex }"
                                :title="reading.latex ? 'LaTeX, typeset by KaTeX' : 'plain text'"
                                @click="toggleLatex(reading)"
                            >
                                {{ reading.latex ? 'TeX' : 'txt' }}
                            </button>
                            <button
                                class="close"
                                title="remove"
                                @click="removeReading(row.node.plain, reading)"
                            >
                                ×
                            </button>
                        </div>
                        <button class="ghost add-read" @click="addReading(row.node.plain)">
                            + reading
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.dev {
    max-width: 1100px;
    margin: 0 auto;
    padding: 0 10px 80px;
}

.head {
    position: sticky;
    top: 0;
    z-index: 3;
    background: var(--dev-bg);
    border-bottom: 1px solid var(--dev-line);
    padding: 6px 0 4px;
    margin-bottom: 2px;
}

.bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    min-height: 28px;
}

.title {
    font-weight: 600;
    font-size: 14px;
}

.title em {
    font-style: normal;
    font-weight: normal;
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--dev-muted);
    border: 1px solid var(--dev-line);
    border-radius: 6px;
    padding: 0 5px;
    margin-left: 2px;
    vertical-align: 1px;
}

.spacer {
    flex: 1;
}

.count,
.status {
    font-size: 11.5px;
    color: var(--dev-muted);
}

.status {
    color: var(--dev-accent);
    text-align: right;
}

button {
    font: inherit;
    font-size: 12px;
    line-height: 1.2;
    padding: 4px 10px;
    border: 1px solid var(--dev-line);
    border-radius: 6px;
    background: #f8f8f8;
    color: inherit;
    cursor: pointer;
}

button:hover:not(:disabled) {
    background: var(--dev-hover);
}

button:active:not(:disabled) {
    background: #b8e8e8;
}

button:disabled {
    opacity: 0.45;
    cursor: default;
}

button.primary {
    background: var(--dev-accent-soft);
    border-color: var(--dev-accent);
}

button.danger {
    color: #b3261e;
}

input {
    font: inherit;
    font-size: 12.5px;
    border: 1px solid var(--dev-line);
    border-radius: 6px;
    padding: 4px 8px;
    background: #fff;
    color: inherit;
}

input:focus {
    outline: none;
    border-color: var(--dev-accent);
    box-shadow: 0 0 0 3px var(--dev-accent-soft);
}

.add-input {
    flex: 1;
    min-width: 150px;
}

.hint {
    margin: 6px 0 10px;
    font-size: 11.5px;
    line-height: 1.55;
    color: var(--dev-muted);
}

.added {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 4px 0;
    padding: 5px 8px;
    border: 1px solid var(--dev-line);
    border-radius: 6px;
    background: #fafafa;
}

.added-label {
    flex: none;
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--dev-muted);
}

.added .term {
    flex: 1;
    min-width: 0;
    cursor: default;
}

.added .term:hover {
    text-decoration: none;
}

.added .desc {
    width: 42%;
    margin: 0;
}

.added .close {
    flex: none;
    padding: 2px 8px;
    line-height: 1;
}

.tree {
    display: flex;
    flex-direction: column;
}

.row {
    display: flex;
    align-items: stretch;
}

.guide {
    position: relative;
    width: 18px;
    flex: none;
}

.guide.vline::before,
.guide.elbow::before {
    content: '';
    position: absolute;
    left: 8px;
    top: 0;
    bottom: 0;
    border-left: 1px solid #dddddd;
}

.guide.elbow.stop::before {
    bottom: calc(100% - 12px);
}

.guide.elbow::after {
    content: '';
    position: absolute;
    left: 8px;
    top: 12px;
    width: 10px;
    border-top: 1px solid #dddddd;
}

.body {
    flex: 1;
    min-width: 0;
    padding: 0 4px;
    border-radius: 4px;
    border-left: 2px solid transparent;
}

.body:hover {
    background: var(--dev-hover);
}

.body.marked {
    background: #fffbee;
    border-left-color: #f0c040;
}

.body.marked:hover {
    background: #fff3cc;
}

.line {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-height: 24px;
    padding-top: 1px;
}

.fold {
    width: 14px;
    flex: none;
    text-align: center;
    cursor: pointer;
    font-size: 10px;
    color: #999999;
    user-select: none;
}

.fold:hover {
    color: var(--dev-fg);
}

.fold.leaf {
    opacity: 0.45;
}

.term {
    flex: 1;
    min-width: 0;
    font-family: var(--mono);
    font-size: 12.5px;
    cursor: pointer;
    overflow-wrap: anywhere;
}

.term:hover {
    text-decoration: underline;
}

.term.limit {
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--dev-accent);
}

.term.grown {
    color: #333333;
}

.when {
    flex: none;
    font-size: 10.5px;
    color: var(--dev-muted);
    white-space: nowrap;
}

.star {
    flex: none;
    border: 0;
    background: transparent;
    padding: 0 6px;
    font-size: 15px;
    line-height: 1;
    border-radius: 4px;
    color: #c9ccd2;
    cursor: pointer;
}

.star:hover {
    background: #fff0c0;
    color: #e8a300;
}

.star.on {
    color: #e8a300;
}

.desc {
    display: block;
    width: calc(100% - 20px);
    margin: 0 0 5px 20px;
    font-size: 12px;
    padding: 2px 6px;
    border-color: transparent;
    background: rgba(0, 0, 0, 0.035);
}

.desc:hover {
    border-color: var(--dev-line);
    background: #fff;
}

.desc:focus {
    background: #fff;
}

.modes {
    display: inline-flex;
    border: 1px solid var(--dev-line);
    border-radius: 6px;
    overflow: hidden;
}

.modes button {
    border: 0;
    border-radius: 0;
    background: transparent;
    padding: 4px 9px;
}

.modes button + button {
    border-left: 1px solid var(--dev-line);
}

.modes button:hover:not(.active) {
    background: var(--dev-hover);
}

.modes button.active {
    background: var(--dev-accent-soft);
    color: var(--dev-accent);
    font-weight: 600;
}

.readings {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 10px;
    margin: 0 0 6px 20px;
    font-size: 11.5px;
}

.readings .baked {
    font-family: var(--mono);
    color: var(--dev-muted);
}

.readings .notation {
    margin-right: 4px;
    font-size: 9.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #a0a4ad;
}

.read-edit {
    display: flex;
    align-items: center;
    gap: 4px;
}

.read-label {
    width: 80px;
}

.read-term {
    width: 230px;
    font-family: var(--mono);
}

.readings .mini {
    padding: 0 3px;
    margin-left: 2px;
    border: 0;
    background: transparent;
    font-size: 11px;
    line-height: 1;
    color: #b0b4bd;
    cursor: pointer;
}

.readings .mini:hover {
    color: var(--dev-accent);
    background: transparent;
}

.read-tex {
    padding: 2px 6px;
    font-size: 10px;
    color: var(--dev-muted);
}

.read-tex.on {
    border-color: var(--dev-accent);
    color: var(--dev-accent);
}

.add-read {
    padding: 2px 7px;
    font-size: 11px;
}

@media (max-width: 640px) {
    .when {
        display: none;
    }

    .hint {
        margin: 4px 0 8px;
    }
}
</style>
