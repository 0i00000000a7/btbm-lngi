<script setup lang="ts" vapor>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import katex from 'katex';
import {
    DISPLAY_MODES,
    START_MS,
    bocfLatex,
    lngiStep,
    milestoneRows,
    dbms,
    type MilestoneRow,
    oneY,
    renderOrdinal,
    veblenLatex,
    zeroY,
    type DisplayMode,
} from './core/wasm';
import { PRE_START_MS, formatDuration, formatMoment, useClock } from './composables/useClock';
import { translate } from './core/i18n';
import { DESC_ZH } from './core/descriptions.zh';
import { MIN_PANEL_HEIGHT, useSettings } from './composables/useSettings';

const clock = useClock();
const settings = useSettings();

/** `import.meta` is not usable in the template, so pull it out in the script. */
const isDev = import.meta.env.DEV;

const started = computed(() => clock.elapsedMs.value >= 0);

/** The computed ordinal (canonical plain form) plus progress. Mode-independent. */
const step = computed(() => lngiStep(clock.elapsedMs.value));

/** Column cap, clamped to a non-negative integer (`0` = no cap). */
const maxColumns = computed(() => Math.max(0, Math.floor(settings.maxColumns.value) || 0));

/**
 * Rendered output, produced on demand.
 *
 * Only the modes actually on screen are rendered — the hero's mode plus every
 * panel's mode — so an unused structure costs nothing. Computation happened
 * once in `step`; this is purely the rendering step.
 *
 * `renderOrdinal`'s third argument selects LaTeX instead of HTML. It is `false`
 * on purpose: these ordinals are brackets, digits and superscripts, which the
 * HTML form renders fine and about 2.5x faster. (KaTeX is still a dependency —
 * should a future notation actually need real math typesetting, pass `true`
 * here and pipe the result through `katex.renderToString`.)
 */
/**
 * The converted notations, typeset by KaTeX. Each one is a rewrite of the
 * BTBMS term into a weaker system, so each has a ceiling: Veblen stops at the
 * Bachmann–Howard ordinal, BOCF at the collapse limit. The fallback chain runs
 * BTBMS → BOCF → Veblen, so a term past a notation's reach is shown in the next
 * one down, and past BOCF's reach it is shown as ordinary BTBMS.
 *
 * KaTeX is expensive, so it is used *only* for the converted term: those are
 * short (a hundred characters at most, capped in the core) and exist only at
 * the start of the timeline. The fallback never goes through KaTeX — doing that
 * for a long ordinal is what once dropped this app from 40 tps to 14.
 *
 * One slot per mode is enough: the ordinal changes every frame, so a wider
 * cache would never be hit, but two panels may show two different modes.
 */
const mathMemo = new Map<DisplayMode, { plain: string; html: string }>();

function renderConverted(plain: string, mode: DisplayMode, cols: number): string {
    const hit = mathMemo.get(mode);
    if (hit && hit.plain === plain) return hit.html;

    const latex =
        mode === 'veblen' ? (veblenLatex(plain) ?? bocfLatex(plain)) : bocfLatex(plain);
    const html =
        latex === null
            ? renderOrdinal(plain, 'index', false, cols)
            : katex.renderToString(latex, { output: 'html', throwOnError: false, strict: false });
    mathMemo.set(mode, { plain, html });
    return html;
}

/**
 * Before the start the walk sits on the empty matrix. The OCF forms print the
 * zero; the matrix-shaped notations show the empty-set sign instead, which is
 * what an empty matrix is — and is not a dash, which would read as "unknown".
 */
const EMPTY_HTML = '<span class="empty-ordinal">∅</span>';
const ZERO_HTML = katex.renderToString('0', {
    output: 'html',
    throwOnError: false,
    strict: false,
});
const ZERO_TEXT = '0';

/**
 * Every mode is a rewrite of the same term, and every rewrite has a ceiling.
 * Past it the BTBMS form is shown instead of a wrong answer.
 */
function renderMode(mode: DisplayMode, plain: string, cols: number): string {
    switch (mode) {
        case 'bocf':
        case 'veblen':
            return renderConverted(plain, mode, cols);
        case 'zeroy':
            return zeroY(plain) ?? renderOrdinal(plain, 'index', false, cols);
        case 'dbms':
            return dbms(plain) ?? renderOrdinal(plain, 'index', false, cols);
        case 'y1':
            return oneY(plain) ?? renderOrdinal(plain, 'index', false, cols);
        default:
            return renderOrdinal(plain, mode, false, cols);
    }
}

const htmlByMode = computed<Record<DisplayMode, string>>(() => {
    if (!started.value) {
        return {
            index: EMPTY_HTML,
            layer: EMPTY_HTML,
            annotated: EMPTY_HTML,
            bocf: ZERO_HTML,
            veblen: ZERO_HTML,
            zeroy: EMPTY_HTML,
            dbms: EMPTY_HTML,
            y1: EMPTY_HTML,
        };
    }

    const out: Record<DisplayMode, string> = {
        index: '',
        layer: '',
        annotated: '',
        bocf: '',
        veblen: '',
        zeroy: '',
        dbms: '',
        y1: '',
    };
    const plain = step.value.ordinal;
    const cols = maxColumns.value;

    const needed = new Set<DisplayMode>([heroMode.value]);
    for (const panel of settings.panels.value) {
        if (!panel.collapsed) needed.add(panel.mode);
    }

    for (const mode of needed) {
        out[mode] = renderMode(mode, plain, cols);
    }
    return out;
});

const main = computed(() => ({
    html: htmlByMode.value[heroMode.value],
    percent: step.value.percent,
}));

/** Before the start point the ordinal is pinned to (0); only show the remaining time here. */
const elapsedText = computed(() =>
    started.value
        ? formatDuration(clock.elapsedMs.value, settings.lang.value)
        : `${formatDuration(-clock.elapsedMs.value, settings.lang.value)} ${t('until start')}`,
);

/**
 * The rewind control is a stack of sliders, one per granularity — the way an
 * odometer reads. Each level carries only its own digit, so "3 days 7 hours" is
 * set on two independent tracks instead of one hair-trigger control.
 *
 * Levels are chosen by the user: the largest selected one is unbounded (up to
 * the available history), the rest wrap at their natural base.
 */
const REWIND_UNITS = [
    { id: 'millennium', label: 'millennia', ms: 1000 * 365 * 86_400_000, base: Number.POSITIVE_INFINITY },
    { id: 'century', label: 'centuries', ms: 100 * 365 * 86_400_000, base: 10 },
    { id: 'decade', label: 'decades', ms: 10 * 365 * 86_400_000, base: 10 },
    { id: 'year', label: 'years', ms: 365 * 86_400_000, base: 10 },
    { id: 'day', label: 'days', ms: 86_400_000, base: 365 },
    { id: 'hour', label: 'hours', ms: 3_600_000, base: 24 },
    { id: 'minute', label: 'minutes', ms: 60_000, base: 60 },
    { id: 'second', label: 'seconds', ms: 1000, base: 60 },
] as const;

/** Selected levels, largest first. */
const activeUnits = computed(() =>
    REWIND_UNITS.filter((u) => settings.rewindUnits.value.includes(u.id)),
);

function isUnitActive(id: string) {
    return settings.rewindUnits.value.includes(id);
}

function toggleUnit(id: string) {
    const current = settings.rewindUnits.value;
    const next = current.includes(id)
        ? current.filter((u) => u !== id)
        : [...current, id];
    // At least one level has to remain, otherwise there is no control at all.
    if (next.length === 0) return;
    settings.rewindUnits.value = next;
}

/** The current rewind, split into one digit per active level. */
const rewindDigits = computed(() => {
    let rest = clock.rewindMs.value;
    const out: Record<string, number> = {};
    for (const unit of activeUnits.value) {
        const digit = Math.floor(rest / unit.ms);
        out[unit.id] = digit;
        rest -= digit * unit.ms;
    }
    return out;
});

/** Range of a level: the largest one is limited only by the available history. */
function maxFor(index: number): number {
    const unit = activeUnits.value[index];
    if (index === 0) {
        return Math.max(0, Math.floor(clock.maxRewindMs.value / unit.ms));
    }
    return Math.min(unit.base, Math.floor(clock.maxRewindMs.value / unit.ms));
}

/** Track fill for a reel, as a CSS custom property the slider's track reads. */
function fillStyle(index: number): Record<string, string> {
    const max = maxFor(index);
    const value = rewindDigits.value[activeUnits.value[index].id] ?? 0;
    const pct = max > 0 ? Math.round((value / max) * 100) : 0;
    return { '--fill': `${pct}%` };
}

function onReel(index: number, event: Event) {
    const digit = Number((event.target as HTMLInputElement).value);
    let total = 0;
    activeUnits.value.forEach((u, i) => {
        total += (i === index ? digit : rewindDigits.value[u.id]) * u.ms;
    });
    clock.setRewind(total);
}

const rewinding = computed(() => clock.rewindMs.value > 0);

/**
 * The jump field is a plain text input — a native `datetime-local` brings the
 * browser's own widget and look, which does not match the rest of the card.
 * The stamp is `YYYY-MM-DD HH:mm:ss` in local time.
 */
function formatStamp(ms: number): string {
    const d = new Date(ms);
    const p = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

/**
 * The inverse of [`formatStamp`], deliberately loose: it takes the first
 * date-like run anywhere in the text, so `公元前209年7月，大泽乡起义` parses just
 * as well as a bare timestamp. Years may be BC (`公元前`, `BC`, or a leading
 * `-`), and the time of day, month and day are all optional.
 */
function parseStamp(text: string): number | null {
    // Chinese keyboards produce full-width digits, dashes, colons and spaces.
    const s = text
        .replace(/[０-９]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0))
        .replace(/[：]/g, ':')
        .replace(/[－—–]/g, '-')
        .replace(/[／]/g, '/')
        .replace(/[．]/g, '.')
        .replace(/[\u3000]/g, ' ');

    const build = (
        era: string | undefined,
        y: number,
        mo: number,
        d: number,
        h = 0,
        mi = 0,
        sec = 0,
    ): number | null => {
        if (!(mo >= 1 && mo <= 12 && d >= 1 && d <= 31 && h <= 23 && mi <= 59 && sec <= 59)) {
            return null;
        }
        // `公元前209年` is astronomical year -208.
        const year = era === '公元前' || era === 'BC' ? -(y - 1) : y;
        // The constructor maps years 0-99 to the 1900s, so set the year after.
        const date = new Date(2000, 0, 1, 0, 0, 0, 0);
        date.setFullYear(year, mo - 1, d);
        date.setHours(h, mi, sec, 0);
        const t = date.getTime();
        return Number.isFinite(t) ? t : null;
    };
    const time = String.raw`(?:\s*[T,，]?\s*(\d{1,2}):(\d{2})(?::(\d{2}))?)?`;

    // 公元前209年7月1日 12:30
    const zh = new RegExp(
        `(公元前|公元)?\\s*(\\d{1,7})\\s*年(?:\\s*(\\d{1,2})\\s*月)?(?:\\s*(\\d{1,2})\\s*[日号])?` + time,
    ).exec(s);
    if (zh) {
        return build(zh[1], +zh[2], zh[3] ? +zh[3] : 1, zh[4] ? +zh[4] : 1, +(zh[5] ?? 0), +(zh[6] ?? 0), +(zh[7] ?? 0));
    }

    // 2026-10-31 20:13:33, -0209-07-01, 2026/10/31
    const iso = new RegExp(
        `(公元前|公元|BC|AD)?\\s*(-?\\d{1,7})\\s*[-/.]\\s*(\\d{1,2})\\s*[-/.]\\s*(\\d{1,2})` + time,
    ).exec(s);
    if (iso) {
        return build(iso[1], +iso[2], +iso[3], +iso[4], +(iso[5] ?? 0), +(iso[6] ?? 0), +(iso[7] ?? 0));
    }

    // 2026-10
    const ym = /^\s*(公元前|公元|BC|AD)?\s*(-?\d{1,7})\s*[-/.]\s*(\d{1,2})\s*$/.exec(s);
    if (ym) return build(ym[1], +ym[2], +ym[3], 1);

    // A bare year.
    const y = /^\s*(公元前|公元|BC|AD)?\s*(-?\d{1,7})\s*年?\s*$/.exec(s);
    if (y) return build(y[1], +y[2], 1, 1);

    return null;
}

/**
 * Draft while the field is being edited; `null` means "follow the view". The
 * field would otherwise be rewritten every second and eat the user's typing.
 */
const jumpDraft = ref<string | null>(null);
const jumpValue = computed(() =>
    jumpDraft.value ?? formatStamp(START_MS + clock.elapsedMs.value),
);

function onJumpFocus() {
    jumpDraft.value = jumpValue.value;
}

function onJumpInput(event: Event) {
    jumpDraft.value = (event.target as HTMLInputElement).value;
}

/** Jump to the moment in the field, clamped to the available history. */
function onJumpCommit() {
    const text = jumpDraft.value;
    jumpDraft.value = null;
    if (!text) return;
    const target = parseStamp(text);
    if (target === null) return;
    const now = START_MS + clock.liveMs.value;
    const clamped = Math.min(Math.max(target, START_MS - PRE_START_MS), now);
    clock.setRewind(now - clamped);
}
/**
 * The speed slider is signed and multiplicative: position `v` is
 * `sign(v) * (10^|v| - 1)`, so 0 sits exactly at the centre (time frozen),
 * negative runs the clock backwards, and the useful range stays reachable
 * without a hair-trigger near zero.
 */
const speedExp = computed(() => {
    const s = clock.timeSpeed.value;
    return Math.sign(s) * Math.log10(Math.abs(s) + 1);
});

function setSpeedExp(event: Event) {
    const v = Number((event.target as HTMLInputElement).value);
    const next = Math.sign(v) * (10 ** Math.abs(v) - 1);
    // Outside dev a forward speed may only close a rewind, never pass live.
    if (!isDev && !rewinding.value && next > 1) return;
    clock.timeSpeed.value = next;
}

const speedText = computed(() => {
    const s = clock.timeSpeed.value;
    if (Math.abs(s) < 1e-9) return '0×';
    const a = Math.abs(s);
    const n =
        a < 10
            ? a.toFixed(1)
            : a < 1e6
              ? Math.round(a).toLocaleString('en-US')
              : a.toExponential(1).replace('e+', 'e');
    return `${s < 0 ? '−' : ''}${n}×`;
});

/**
 * Outside dev, acceleration is only a way to catch up: the moment the view
 * reaches live time it switches itself off and settles exactly on live.
 */
watch(clock.elapsedMs, () => {
    if (isDev) return;
    const speed = clock.timeSpeed.value;
    if (speed > 1 && clock.elapsedMs.value >= clock.liveMs.value) {
        clock.timeSpeed.value = 1;
        clock.goLive();
    } else if (speed !== 1 && clock.rewindMs.value >= clock.maxRewindMs.value) {
        // Frozen or reversed all the way into the earliest reachable moment.
        clock.timeSpeed.value = 1;
        clock.setRewind(clock.maxRewindMs.value);
    }
});

const rewindText = computed(() => {
    const lang = settings.lang.value;
    if (rewinding.value) return `${formatDuration(clock.rewindMs.value, lang)} ${t('ago')}`;
    const ahead = clock.aheadMs.value;
    return ahead > 0 ? `${formatDuration(ahead, lang)} ${t('ahead')}` : t('live');
});


/** Landmarks, in order; generated at build time. */
const milestones = milestoneRows();
// Descriptions are authored in `milestones.rs` and shipped inside the wasm, so
// the markup they carry is ours, not user input — `v-html` is safe there.

/**
 * Their arrival times, formatted once. `toLocaleString` is not cheap, and the
 * list re-renders every frame — recomputing 133 of them per frame would cost
 * more than the ordinal itself.
 */
const milestoneDates = computed(() =>
    milestones.map((m) => formatMoment(START_MS + m.atMs, settings.lang.value)),
);

/**
 * True when the time is not a real date — `LIMIT` says "never", and the far
 * end of the chain is aeons out — so the cell reads as a phrase, not a moment.
 */
const isRelative = (atMs: number) =>
    !Number.isFinite(atMs) || Math.abs(START_MS + atMs) > 8.64e15;

/**
 * The landmark's term as HTML, so the caret form the table is written in
 * (`(0)(1^(2))`) shows up as superscripts instead of literal `^` and `(…)`.
 * Zero is the empty matrix — the table stores it as an empty string — so it is
 * spelled out here. Entries that are not matrices (`LIMIT`) stay as text.
 */
function termHtml(ordinal: string): string {
    if (!ordinal) return ZERO_TEXT;
    try {
        return renderOrdinal(ordinal, 'index', false, 0);
    } catch {
        return ordinal.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }
}

const milestoneTerms = milestones.map((m) => termHtml(m.ordinal));

/**
 * SHO arrives at a fixed moment; Auto switches structure there — Layer is the
 * compact form below it, Index stays readable above.
 */
const SHO_MS =
    milestones.find((m) => m.ordinal === '(0)(1^(2))')?.atMs ?? Number.POSITIVE_INFINITY;

/** The hero's structure: `auto` resolves to Layer before SHO, Index after. */
const heroMode = computed<DisplayMode>(() => {
    const setting = settings.defaultMode.value;
    if (setting !== 'auto') return setting;
    return clock.elapsedMs.value < SHO_MS ? 'layer' : 'index';
});

/**
 * Whether the landmark is behind the walk.
 *
 * Zero is special: it is before every moment, so its landmark is always done —
 * even while the walk is still counting down to the start.
 */
const reached = (m: MilestoneRow) => m.ordinal === '' || m.atMs <= clock.elapsedMs.value;

/** The first landmark not yet reached at the *viewed* time. */
const nextMilestone = computed(() => milestones.find((m) => !reached(m)) ?? null);

/**
 * The first landmark the walk has not reached *now*. Rewinding moves
 * `nextMilestone` with the view but leaves this one where live time is, so the
 * list can show both: where you are looking, and where the walk actually is.
 */
const liveNext = computed(() => milestones.find((m) => m.atMs > clock.liveMs.value) ?? null);

const reachedCount = computed(() => milestones.filter(reached).length);

/** `LIMIT` has no arrival time; everything else does. */
const reachable = (atMs: number) => Number.isFinite(atMs);

/** Which landmark the card is showing: the live one, or a clicked one. */
const focused = ref<number | null>(null);

/** Index of the next landmark (the last one once everything is reached). */
const nextIndex = computed(() => {
    const next = nextMilestone.value;
    return next ? milestones.indexOf(next) : milestones.length - 1;
});

const shownIndex = computed(() => focused.value ?? nextIndex.value);
const shown = computed(() => milestones[shownIndex.value] ?? null);
const shownTerm = computed(() => milestoneTerms[shownIndex.value] ?? '');
const shownDate = computed(() =>
    shown.value ? formatMoment(START_MS + shown.value.atMs, settings.lang.value) : '',
);

/** The description in the chosen language, falling back to the English one. */
const shownDesc = computed(() => {
    const m = shown.value;
    if (!m) return '';
    return settings.lang.value === 'zh' ? (DESC_ZH[m.ordinal] ?? m.desc) : m.desc;
});

/** Distance to the shown landmark, in the chosen language. */
const awayText = computed(() => {
    const m = shown.value;
    if (!m) return '';
    const d = formatDuration(m.atMs - clock.elapsedMs.value, settings.lang.value);
    return settings.lang.value === 'zh' ? `还有 ${d}` : `${d} away`;
});

/**
 * Position between the landmark before the shown one and the shown one, 0..1.
 * A reached landmark sits at 1; `LIMIT` has no stretch to measure.
 */
const shownProgress = computed(() => {
    const m = shown.value;
    if (!m || reached(m)) return 1;
    const index = shownIndex.value;
    const from = index > 0 ? milestones[index - 1].atMs : 0;
    const span = m.atMs - from;
    if (!(span > 0)) return 1;
    const ratio = (clock.elapsedMs.value - from) / span;
    return Math.min(1, Math.max(0, ratio));
});

const listEl = ref<HTMLElement | null>(null);

/**
 * Centre a row inside the list. Done by hand rather than with `scrollIntoView`,
 * which would also scroll the page — the card often sits below the fold, and
 * jumping there on load would be hostile.
 */
function scrollToIndex(index: number) {
    const list = listEl.value;
    const row = list?.children[index] as HTMLElement | undefined;
    if (!list || !row) return;
    const box = list.getBoundingClientRect();
    const cell = row.getBoundingClientRect();
    list.scrollTop += cell.top - box.top - (box.height - cell.height) / 2;
}

/** Clicking a row looks at it; clicking it again leaves the list where it is. */
function focusRow(index: number) {
    focused.value = focused.value === index ? null : index;
}

/** Drop the focus and jump the list back to where the walk actually is. */
function goLive() {
    focused.value = null;
    scrollToIndex(nextIndex.value);
}

/**
 * Landing this long before a landmark means its arrival can be watched.
 */
const JUMP_LEAD_MS = 10_000;

/** The live position, in ms since the start. */
const liveMs = computed(() => clock.liveMs.value);

/**
 * Whether a landmark can be jumped to. For everyone the jump only goes
 * backwards — a moment that has not happened yet is not reachable, so the
 * button is not offered rather than silently landing somewhere else. The
 * developer build may land anywhere.
 */
function canJump(atMs: number): boolean {
    return Number.isFinite(atMs) && (isDev || atMs - JUMP_LEAD_MS < liveMs.value);
}

function jumpTo(atMs: number) {
    if (!canJump(atMs)) return;
    const target = atMs - JUMP_LEAD_MS;
    if (isDev) {
        clock.seek(target);
    } else {
        clock.setRewind(liveMs.value - target);
    }
}

// Start at the last landmark already passed rather than at the first ordinal:
// the interesting part of a 133-entry list is where the walk actually is.
onMounted(() => scrollToIndex(Math.max(0, nextIndex.value - 1)));

/** UI text in the chosen language; notation names pass through unchanged. */
const t = (text: string) => translate(settings.lang.value, text);

const modeLabel = (id: DisplayMode | 'auto') => {
    if (id === 'auto') return t('Auto');
    const mode = DISPLAY_MODES.find((m) => m.id === id);
    if (!mode) return id;
    return settings.lang.value === 'zh' ? (mode.labelZh ?? mode.label) : mode.label;
};

/** What the hero's corner says: the resolved structure, with Auto called out. */
const heroModeLabel = computed(() =>
    settings.defaultMode.value === 'auto'
        ? `${t('Auto')} · ${modeLabel(heroMode.value)}`
        : modeLabel(heroMode.value),
);

/**
 * Dropdowns are hand-built: a native `<select>` popup is drawn by the platform
 * and cannot be styled, so the list would not match the rest of the UI.
 * `openSelect` holds the id of the open dropdown (a panel id, or the constant
 * below for the settings modal), so only one can be open at a time.
 */
const SETTINGS_SELECT = 'settings';
const openSelect = ref<string | null>(null);

function toggleSelect(id: string) {
    openSelect.value = openSelect.value === id ? null : id;
}

function pickPanelMode(panel: { mode: DisplayMode }, mode: DisplayMode) {
    panel.mode = mode;
    openSelect.value = null;
}

function pickDefaultMode(mode: DisplayMode | 'auto') {
    settings.defaultMode.value = mode;
    openSelect.value = null;
}

/** Close the open dropdown when the click lands outside any `.select`. */
function closeOnOutside(e: PointerEvent) {
    if (!(e.target instanceof Element) || !e.target.closest('.select')) openSelect.value = null;
}

function closeOnEscape(e: KeyboardEvent) {
    if (e.key === 'Escape') openSelect.value = null;
}

onMounted(() => {
    window.addEventListener('pointerdown', closeOnOutside);
    window.addEventListener('keydown', closeOnEscape);
});
onUnmounted(() => {
    window.removeEventListener('pointerdown', closeOnOutside);
    window.removeEventListener('keydown', closeOnEscape);
});

/** Any box with a writable `height` (panels, the hero ordinal box). */
interface Resizable {
    height: number;
}

/** The hero ordinal box: wrap the `heroHeight` ref into the shape `startResize` wants. */
const heroBox: Resizable = {
    get height() {
        return settings.heroHeight.value;
    },
    set height(v: number) {
        settings.heroHeight.value = v;
    },
};

/** Drag the handle at the bottom of a box to resize it (mirrors `makeResizable`, `ui.js:36`). */
function startResize(box: Resizable, e: PointerEvent) {
    e.preventDefault();
    const startY = e.clientY;
    const startHeight = box.height;
    let frame = 0;

    const move = (ev: PointerEvent) => {
        if (frame) return; // throttle with rAF so we don't write reactive state every pixel
        frame = requestAnimationFrame(() => {
            box.height = Math.max(MIN_PANEL_HEIGHT, startHeight + ev.clientY - startY);
            frame = 0;
        });
    };
    const up = () => {
        if (frame) cancelAnimationFrame(frame);
        frame = 0;
        window.removeEventListener('pointermove', move);
        window.removeEventListener('pointerup', up);
    };

    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
}
</script>

<template>
    <div class="app" :style="{ fontFamily: settings.fontFamily.value, fontSize: settings.fontSize.value + 'px' }">
        <header class="topbar">
            <div class="brand">
                <h1>BTBMS LNGI</h1>
                <span class="tag" :class="{ past: !clock.atLive.value }">{{
                    t(rewinding ? 'reviewing' : clock.aheadMs.value > 0 ? 'ahead' : 'live')
                }}</span>
            </div>
            <div class="top-actions">
                <a v-if="isDev" class="ghost" href="/dev.html">{{ t('Milestones (dev)') }}</a>
                <button class="ghost" @click="settings.showSettings.value = true">{{ t('Settings') }}</button>
            </div>
        </header>

        <section class="hero">
            <div class="hero-label">
                <span>{{ t('Current ordinal') }}</span>
                <span>{{ heroModeLabel }}</span>
            </div>
            <div
                class="hero-ordinal"
                :style="{ height: settings.heroHeight.value + 'px' }"
                v-html="main.html"
            />
            <div
                class="resize-handle"
                :title="t('Drag to resize')"
                @pointerdown="startResize(heroBox, $event)"
            />

            <div class="progress">
                <div class="bar"><div class="fill" :style="{ width: main.percent * 100 + '%' }" /></div>
                <div class="caption">
                    <span><b>{{ (main.percent * 100).toFixed(3) }}%</b> {{ t('to next ordinal') }}</span>
                    <span>{{ elapsedText }}</span>
                </div>
            </div>

        </section>

        <!-- Stay visible while rewound into the pre-start stretch: hiding it there
             would also hide the only way back to live. -->
        <section v-if="started || rewinding || isDev" class="rewind-card">
            <div class="section-head">
                <h2>
                    <button
                        class="fold"
                        :title="t(settings.collapsedCards.value.includes('rewind') ? 'Expand' : 'Collapse')"
                        @click="settings.toggleCard('rewind')"
                    >
                        {{ settings.collapsedCards.value.includes('rewind') ? '▶' : '▼' }}
                    </button>
                    {{ t('Rewind') }}
                </h2>
                <span class="rewind-value">{{ rewindText }}</span>
            </div>
            <template v-if="!settings.collapsedCards.value.includes('rewind')">
            <div class="rewind-jump">
                <input
                    type="text"
                    class="jump-field"
                    spellcheck="false"
                    autocomplete="off"
                    placeholder="YYYY-MM-DD HH:mm:ss"
                    :value="jumpValue"
                    @focus="onJumpFocus"
                    @input="onJumpInput"
                    @change="onJumpCommit"
                    @blur="onJumpCommit"
                />
                <span>{{ t('jump to a moment') }}</span>
            </div>
            <div class="rewind-levels">
                <button
                    v-for="unit in REWIND_UNITS"
                    :key="unit.id"
                    class="ghost chip"
                    :class="{ on: isUnitActive(unit.id) }"
                    @click="toggleUnit(unit.id)"
                >
                    {{ t(unit.label) }}
                </button>
                <button class="ghost" :disabled="clock.atLive.value" @click="clock.goLive()">
                    {{ t('Back to live') }}
                </button>
            </div>
            <div class="rewind-reels">
                <label v-for="(unit, index) in activeUnits" :key="unit.id" class="reel">
                    <span class="reel-label">{{ t(unit.label) }}</span>
                    <input
                        type="range"
                        min="0"
                        :max="maxFor(index)"
                        :value="rewindDigits[unit.id]"
                        :style="fillStyle(index)"
                        @input="onReel(index, $event)"
                    />
                    <span class="reel-value">{{ rewindDigits[unit.id] }}</span>
                </label>
            </div>
            <div class="rewind-row">
                <span>{{ t('Each slider is one digit; pick the granularities you want above.') }}</span>
            </div>
            <div class="rewind-speed" :title="t('0× freezes time; negative runs it backwards')">
                <span>{{ t('speed') }}</span>
                <input
                    type="range"
                    min="-8"
                    max="8"
                    step="0.05"
                    :value="speedExp"
                    @input="setSpeedExp"
                />
                <span class="speed-value">{{ speedText }}</span>
            </div>

            <div v-if="isDev" class="rewind-dev">
                <span class="rewind-dev-tag">{{ t('dev') }}</span>
                <label>
                    {{ t('offset (s)') }}
                    <input
                        type="number"
                        step="1"
                        :value="clock.timeOffsetMs.value / 1000"
                        @input="clock.timeOffsetMs.value = Number(($event.target as HTMLInputElement).value) * 1000"
                    />
                </label>
                <button class="ghost" @click="clock.reset()">{{ t('Reset time') }}</button>
            </div>
            </template>
        </section>

        <section v-if="milestones.length" class="milestone-card">
            <div class="section-head">
                <h2>
                    <button
                        class="fold"
                        :title="t(settings.collapsedCards.value.includes('milestones') ? 'Expand' : 'Collapse')"
                        @click="settings.toggleCard('milestones')"
                    >
                        {{ settings.collapsedCards.value.includes('milestones') ? '▶' : '▼' }}
                    </button>
                    {{ t('Milestones') }}
                </h2>
                <span class="rewind-value">{{ reachedCount }} / {{ milestones.length }}</span>
            </div>
            <template v-if="!settings.collapsedCards.value.includes('milestones')">
            <div v-if="shown" class="milestone-next">
                <div class="milestone-term" v-html="shownTerm" />
                <div class="milestone-side">
                    <span v-if="shown.title" class="milestone-title">{{ shown.title }}</span>
                    <span
                        class="milestone-when"
                        :class="{ relative: shown && isRelative(shown.atMs) }"
                    >{{ shownDate }}</span>
                    <button
                        v-if="focused !== null"
                        class="ghost milestone-live"
                        :title="t('Back to where the walk is')"
                        @click="goLive"
                    >
                        {{ t('live') }}
                    </button>
                </div>
            </div>
            <div v-if="focused !== null && canJump(shown.atMs)" class="milestone-actions">
                <button class="ghost" @click="jumpTo(shown.atMs)">
                    {{ t('↩ jump to 10 seconds before this landmark') }}
                </button>
            </div>
            <p v-if="shownDesc" class="milestone-desc" v-html="shownDesc" />
            <div class="bar"><div class="fill" :style="{ width: shownProgress * 100 + '%' }" /></div>
            <div class="milestone-foot">
                <template v-if="focused === null && !nextMilestone">
                    <span>{{ t('Every landmark reached. The sequence keeps going.') }}</span>
                </template>
                <template v-else-if="shown && !reachable(shown.atMs)">
                    <span>{{ t('unreachable — the walk never gets there') }}</span>
                </template>
                <template v-else-if="shown">
                    <span>{{ awayText }}</span>
                    <span>{{ (shownProgress * 100).toFixed(2) }}% {{ t('of this stretch') }}</span>
                </template>
            </div>

            <ol ref="listEl" class="milestone-list">
                <li
                    v-for="(m, index) in milestones"
                    :key="m.ordinal"
                    class="milestone-row"
                    :class="{
                        reached: reached(m),
                        next: m === nextMilestone,
                        'live-next': m === liveNext,
                        focused: focused === index,
                    }"
                    :title="t(focused === index ? 'Click to follow the walk again' : 'Click to look at this landmark')"
                    @click="focusRow(index)"
                >
                    <span class="milestone-index">{{ index + 1 }}</span>
                    <span class="milestone-row-term">
                        <span v-html="milestoneTerms[index]" />
                        <span v-if="m.title" class="milestone-row-title">{{ m.title }}</span>
                    </span>
                    <span
                        class="milestone-row-date"
                        :class="{ relative: isRelative(m.atMs) }"
                        >{{ milestoneDates[index] }}</span
                    >
                    <span class="milestone-row-mark">{{ reached(m) ? '✓' : '' }}</span>
                </li>
            </ol>
            </template>
        </section>

        <section class="stats">
            <div class="stat">
                <div class="label">{{ t('Running at') }}</div>
                <div class="value">{{ clock.tps.value.toFixed(1) }} tps</div>
            </div>
        </section>

        <section>
            <div class="section-head">
                <h2>{{ t('Panels') }}</h2>
                <button class="ghost" @click="settings.addPanel(heroMode)">{{ t('+ Add panel') }}</button>
            </div>
            <div class="panels">
                <div
                    v-for="panel in settings.panels.value"
                    :key="panel.id"
                    class="panel"
                    :style="{ flexBasis: `calc(${panel.width}% - 12px)`, '--hue': panel.hue }"
                >
                    <div class="panel-head">
                        <button
                            class="fold"
                            :title="t(panel.collapsed ? 'Expand' : 'Collapse')"
                            @click="settings.togglePanel(panel.id)"
                        >
                            {{ panel.collapsed ? '▶' : '▼' }}
                        </button>
                        <div class="select">
                            <button
                                type="button"
                                class="select-trigger"
                                :aria-expanded="openSelect === panel.id"
                                @click="toggleSelect(panel.id)"
                            >
                                <span class="select-value">{{ modeLabel(panel.mode) }}</span>
                            </button>
                            <ul v-if="openSelect === panel.id" class="select-menu">
                                <li v-for="m in DISPLAY_MODES" :key="m.id">
                                    <button
                                        type="button"
                                        class="select-option"
                                        :class="{ selected: m.id === panel.mode }"
                                        @click="pickPanelMode(panel, m.id)"
                                    >
                                        {{ modeLabel(m.id) }}
                                    </button>
                                </li>
                            </ul>
                        </div>
                        <button class="ghost" @click="settings.removePanel(panel.id)">{{ t('Remove') }}</button>
                    </div>
                    <div
                        v-if="!panel.collapsed"
                        class="panel-body"
                        :style="{ height: panel.height + 'px' }"
                        v-html="htmlByMode[panel.mode]"
                    />
                    <div
                        v-if="!panel.collapsed"
                        class="resize-handle"
                        :title="t('Drag to resize')"
                        @pointerdown="startResize(panel, $event)"
                    />
                </div>
            </div>
        </section>

        <div v-if="settings.showSettings.value" class="modal-backdrop" @click.self="settings.showSettings.value = false">
            <div class="modal">
                <h3>{{ t('Settings') }}</h3>
                <label>
                    {{ t('Font family') }}
                    <input v-model="settings.fontFamily.value" />
                </label>
                <label>
                    {{ t('Font size') }}
                    <input type="number" min="8" max="48" v-model.number="settings.fontSize.value" />
                </label>
                <label>
                    {{ t('Language') }}
                    <div class="lang-row">
                        <button
                            type="button"
                            class="ghost chip"
                            :class="{ on: settings.lang.value === 'en' }"
                            @click="settings.lang.value = 'en'"
                        >
                            English
                        </button>
                        <button
                            type="button"
                            class="ghost chip"
                            :class="{ on: settings.lang.value === 'zh' }"
                            @click="settings.lang.value = 'zh'"
                        >
                            中文
                        </button>
                    </div>
                </label>
                <label>
                    {{ t('Default display mode') }}
                    <div class="select">
                        <button
                            type="button"
                            class="select-trigger"
                            :aria-expanded="openSelect === SETTINGS_SELECT"
                            @click="toggleSelect(SETTINGS_SELECT)"
                        >
                            <span class="select-value">{{ modeLabel(settings.defaultMode.value) }}</span>
                        </button>
                        <ul v-if="openSelect === SETTINGS_SELECT" class="select-menu">
                            <li>
                                <button
                                    type="button"
                                    class="select-option"
                                    :class="{ selected: settings.defaultMode.value === 'auto' }"
                                    @click="pickDefaultMode('auto')"
                                >
                                    {{ t('Auto') }}
                                </button>
                            </li>
                            <li v-for="m in DISPLAY_MODES" :key="m.id">
                                <button
                                    type="button"
                                    class="select-option"
                                    :class="{ selected: m.id === settings.defaultMode.value }"
                                    @click="pickDefaultMode(m.id)"
                                >
                                    {{ modeLabel(m.id) }}
                                </button>
                            </li>
                        </ul>
                    </div>
                </label>

                <label>
                    {{ t('Columns shown (0 = no limit)') }}
                    <input type="number" min="0" step="1" v-model.number="settings.maxColumns.value" />
                </label>
                <p class="hint">
                    {{ t('Extra columns are replaced by an ellipsis. Rendering cost grows with the column count, so a cap keeps the frame rate steady.') }}
                </p>

                <button class="primary" @click="settings.showSettings.value = false">{{ t('Close') }}</button>
            </div>
        </div>
    </div>
</template>
