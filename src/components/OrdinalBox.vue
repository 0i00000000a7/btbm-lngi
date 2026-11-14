<script setup lang="ts" vapor>
import { computed, onMounted, onUnmounted, ref, watchEffect } from 'vue';
import { writeOrdinal } from '../core/dom';
import { OrdinalPainter, parseRuns } from '../core/ordinal-canvas';
import { idleHtml, renderMode } from '../core/render';
import type { DisplayMode } from '../core/wasm';

/**
 * A fixed-height ordinal box.
 *
 * A panel renders itself instead of being rendered by the page: while the box
 * is scrolled out of view it does nothing at all, which is the point — every
 * visible box has to redraw on every frame, so an invisible one must not.
 *
 * Drawing goes through a canvas (see `core/ordinal-canvas`), which keeps the
 * browser from parsing and laying out a few thousand characters per frame. A
 * panel scrolled part-way down only pays for the lines on screen: the canvas
 * is pinned to the top-left of the box and a tail element provides the scroll
 * range, in both directions once the text is magnified.
 *
 * Magnifying does not re-wrap the text. The layout is always the box's own
 * width, so zooming in shows fewer characters and pans, like magnifying a
 * picture — the glyphs get bigger, the lines do not move. The one thing that
 * does follow the zoom is the floor under the nested levels (see
 * `core/ordinal-canvas`): they are laid out smaller as the box is magnified,
 * so the shape of the ordinal is the same at every zoom.
 *
 * Markup the painter does not understand — KaTeX output, the empty-set sign —
 * is handed back to the DOM, where it is written by [`writeOrdinal`].
 */
const props = defineProps<{
    mode: DisplayMode;
    plain: string;
    /** The curve value `plain` was produced from; the pinned table is keyed by it. */
    value: number;
    cols: number;
    started: boolean;
    height: number;
    /** Magnification of the drawn text; the layout is always the box width. */
    scale: number;
    /** Changes whenever the font settings do, so the painter re-reads them. */
    styleKey: string;
}>();

const emit = defineEmits<{ (e: 'scale', value: number): void }>();

const box = ref<HTMLElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
const dom = ref<HTMLElement | null>(null);
const tail = ref<HTMLElement | null>(null);

const visible = ref(false);

const html = computed(() =>
    props.started
        ? renderMode(props.mode, props.plain, props.cols, props.value)
        : idleHtml(props.mode),
);

let painter: OrdinalPainter | null = null;
/** What the last paint drew; a frame that changes none of it is skipped. */
let painted = '';
/**
 * Whether the painter's copy of the typography is out of date.
 *
 * Reading it means `getComputedStyle`, which forces a style and layout flush —
 * a fifth of the frame in a phone profile — so it is read when the box changes
 * shape or the font settings change, not on every frame.
 */
let styleStale = true;
let styleKey = '';

/**
 * The content point a gesture is zooming around, and where it sat in the box.
 *
 * The scroll that keeps it there can only be applied once the tail has grown
 * to the new size — before that the browser clamps it — so `draw` does it.
 */
let anchor: { x: number; y: number; dx: number; dy: number } | null = null;

function draw(): void {
    const node = canvas.value;
    const bx = box.value;
    if (!node || !bx || !visible.value) return;

    if (props.styleKey !== styleKey) {
        styleKey = props.styleKey;
        styleStale = true;
    }

    const source = html.value;
    const runs = parseRuns(source);

    if (runs === null) {
        if (node.style.display !== 'none') {
            node.style.display = 'none';
            if (dom.value) dom.value.style.display = 'block';
            if (tail.value) tail.value.style.height = '0px';
        }
        // The canvas keeps the last ordinal it drew, so whatever comes next has
        // to be painted even if it is the same string.
        painted = '';
        if (dom.value) writeOrdinal(dom.value, source);
        return;
    }

    const ctx = node.getContext('2d');
    if (!ctx) return;
    if (painter === null) {
        painter = new OrdinalPainter(ctx);
        styleStale = true;
    }
    if (styleStale) {
        painter.syncStyle(bx);
        styleStale = false;
    }

    // Fractional on purpose: a panel's width is a percentage of the row, and
    // rounding it would leave the canvas a fraction of a pixel off the box.
    const rect = bx.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) return;

    // Size the backing store to whole device pixels and let the CSS size
    // follow from it, so one backing pixel is exactly one device pixel. Any
    // other pairing makes the browser resample the bitmap, which is what turns
    // the text soft.
    const dpr = window.devicePixelRatio || 1;
    const pixelWidth = Math.max(1, Math.round(rect.width * dpr));
    const pixelHeight = Math.max(1, Math.round(rect.height * dpr));
    const width = pixelWidth / dpr;
    const height = pixelHeight / dpr;
    const scale = props.scale;

    const lines = painter.layout(source, runs, width, scale);
    const tailWidth = width * scale;
    const tailHeight = Math.max(0, lines.length * painter.lineHeightPx * scale - height);
    const tailEl = tail.value;
    if (tailEl) {
        const w = `${tailWidth}px`;
        const h = `${tailHeight}px`;
        if (tailEl.style.width !== w) tailEl.style.width = w;
        if (tailEl.style.height !== h) tailEl.style.height = h;
    }

    if (anchor !== null) {
        const pending = anchor;
        anchor = null;
        bx.scrollLeft = pending.x * scale - pending.dx;
        bx.scrollTop = pending.y * scale - pending.dy;
    }

    const key = `${source}\u0000${pixelWidth}\u0000${pixelHeight}\u0000${scale}\u0000${bx.scrollLeft}\u0000${bx.scrollTop}`;
    if (key === painted) return;
    painted = key;

    if (node.style.display === 'none') {
        node.style.display = 'block';
        if (dom.value) dom.value.style.display = 'none';
    }
    if (node.width !== pixelWidth || node.height !== pixelHeight) {
        node.width = pixelWidth;
        node.height = pixelHeight;
        node.style.width = `${width}px`;
        node.style.height = `${height}px`;
    }

    painter.paint(lines, bx.scrollLeft, bx.scrollTop, width, height, dpr, scale);
}

/** Magnify around a point of the box, in box coordinates. */
function zoomAt(dx: number, dy: number, next: number): void {
    const bx = box.value;
    if (!bx || next === props.scale) return;
    anchor = {
        x: (bx.scrollLeft + dx) / props.scale,
        y: (bx.scrollTop + dy) / props.scale,
        dx,
        dy,
    };
    emit('scale', next);
}

/**
 * Ctrl + wheel, and the two-finger trackpad gesture that arrives as one.
 *
 * A plain wheel scrolls the box, which is what a reader expects over text; the
 * browser's own page zoom would blur the canvas, so it is suppressed.
 */
function onWheel(e: WheelEvent): void {
    if (!e.ctrlKey && !e.metaKey) return;
    const bx = box.value;
    if (!bx) return;
    e.preventDefault();
    const rect = bx.getBoundingClientRect();
    zoomAt(e.clientX - rect.left, e.clientY - rect.top, props.scale * Math.exp(-e.deltaY * 0.002));
}

/** Two fingers on the box magnify it; one finger still pans it natively. */
let pinch: { distance: number; scale: number } | null = null;

function touchSpan(e: TouchEvent): number {
    const [a, b] = [e.touches[0], e.touches[1]];
    return Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
}

function onTouchStart(e: TouchEvent): void {
    pinch =
        e.touches.length === 2 && touchSpan(e) > 0
            ? { distance: touchSpan(e), scale: props.scale }
            : null;
}

function onTouchMove(e: TouchEvent): void {
    if (e.touches.length !== 2 || pinch === null) return;
    const bx = box.value;
    if (!bx) return;
    e.preventDefault();
    const rect = bx.getBoundingClientRect();
    const [a, b] = [e.touches[0], e.touches[1]];
    zoomAt(
        (a.clientX + b.clientX) / 2 - rect.left,
        (a.clientY + b.clientY) / 2 - rect.top,
        pinch.scale * (touchSpan(e) / pinch.distance),
    );
}

function onTouchEnd(e: TouchEvent): void {
    if (e.touches.length < 2) pinch = null;
}

let observer: IntersectionObserver | undefined;
let resizer: ResizeObserver | undefined;

onMounted(() => {
    const node = box.value;
    // Without an observer (or a layout engine) there is nothing to gate on.
    if (!node || typeof IntersectionObserver === 'undefined') {
        visible.value = true;
        return;
    }
    observer = new IntersectionObserver(
        (entries) => {
            visible.value = entries[entries.length - 1].isIntersecting;
        },
        // A margin so a panel is already drawn by the time it scrolls in.
        { rootMargin: '300px' },
    );
    // The observer's first callback arrives a frame later; draw now so a panel
    // that is already on screen does not start out blank.
    visible.value = node.getBoundingClientRect().top < window.innerHeight + 300;
    observer.observe(node);

    resizer = new ResizeObserver(() => {
        styleStale = true;
        draw();
    });
    resizer.observe(node);

    // The palette follows the system theme, which can change under us.
    const theme = window.matchMedia?.('(prefers-color-scheme: dark)');
    theme?.addEventListener?.('change', onThemeChange);
});

function onThemeChange(): void {
    styleStale = true;
    draw();
}

onUnmounted(() => {
    observer?.disconnect();
    resizer?.disconnect();
    window.matchMedia?.('(prefers-color-scheme: dark)')?.removeEventListener?.('change', onThemeChange);
});

// `post` so the box has its new height before the canvas is sized from it.
watchEffect(draw, { flush: 'post' });
</script>

<template>
    <div
        ref="box"
        class="ordinal-box"
        :style="{ height: height + 'px' }"
        @scroll="draw"
        @wheel="onWheel"
        @touchstart="onTouchStart"
        @touchmove="onTouchMove"
        @touchend="onTouchEnd"
        @touchcancel="onTouchEnd"
    >
        <canvas ref="canvas" class="ordinal-canvas" />
        <span ref="dom" class="ordinal-dom" />
        <div ref="tail" class="ordinal-tail" />
    </div>
</template>
