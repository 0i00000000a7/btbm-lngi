import { computed, onUnmounted, ref } from 'vue';
import { START_MS } from '../core/wasm';
import { type Lang, translate } from '../core/i18n';

/**
 * How far before the start the view may be pushed.
 *
 * Nothing changes back there — the walk is pinned to the empty matrix — but the
 * region has to be reachable, otherwise the rewind stops dead at the first
 * moment of the timeline. A hundred millennia is far enough for any date a
 * `Date` can hold, and keeps the year reel's range in the thousands.
 */
export const PRE_START_MS = 100_000 * 365 * 86_400_000;

/**
 * A clock synchronized with the wall clock.
 *
 * - On load, `virtualElapsed = Date.now() - START_MS` (may be negative -> not started);
 * - Afterwards, each frame does `virtualElapsed += dt * timeSpeed`;
 * - Externally, `elapsedMs = virtualElapsed + timeOffsetMs`.
 *
 * `driftMs` accumulates `dt * (timeSpeed - 1)`, so the distance from the live
 * position is `timeOffsetMs + driftMs`: accelerating the clock moves the view
 * forward exactly as a positive offset would, and is reported as such.
 */
export function useClock() {
    const virtualElapsed = ref(Date.now() - START_MS);
    const timeSpeed = ref(1);
    const timeOffsetMs = ref(0);
    /**
     * How far virtual time has run ahead of the wall clock. Running the clock
     * fast is the same thing as being handed a growing offset, so it is counted
     * as one: a view pushed forward by acceleration is not "live".
     */
    const driftMs = ref(0);
    const elapsedMs = ref(virtualElapsed.value);
    const tps = ref(0);

    let raf = 0;
    let last = performance.now();
    let frames = 0;
    let acc = 0;

    const tick = (now: number) => {
        const dt = now - last;
        last = now;
        virtualElapsed.value += dt * timeSpeed.value;
        driftMs.value += dt * (timeSpeed.value - 1);
        elapsedMs.value = virtualElapsed.value + timeOffsetMs.value;

        frames += 1;
        acc += dt;
        if (acc >= 500) {
            tps.value = (frames * 1000) / acc;
            frames = 0;
            acc = 0;
        }
        raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    onUnmounted(() => cancelAnimationFrame(raf));

    /** The live position, in ms since the start. */
    const liveMs = computed(() => Math.max(0, virtualElapsed.value - driftMs.value));

    /**
     * How far the view can be pushed back: everything up to now, plus the run-up
     * before the start.
     */
    const maxRewindMs = computed(() => liveMs.value + PRE_START_MS);

    /** Signed distance from the live position: negative behind, positive ahead. */
    const offsetMs = computed(() => timeOffsetMs.value + driftMs.value);

    /** How far into the past the view currently sits (`0` = not behind). */
    const rewindMs = computed(() => Math.max(0, -offsetMs.value));

    /** How far into the future acceleration has pushed the view. */
    const aheadMs = computed(() => Math.max(0, offsetMs.value));

    /** Whether the view sits exactly on the live position. */
    const atLive = computed(() => Math.abs(offsetMs.value) < 1);

    /**
     * Show an earlier moment. Only backwards: a positive offset would put the
     * view in the future, which the live clock has not reached yet.
     */
    function setRewind(ms: number) {
        const back = Math.min(Math.max(ms, 0), maxRewindMs.value);
        timeOffsetMs.value = -back - driftMs.value;
    }

    /** Snap back to the live position, cancelling both the rewind and the drift. */
    function goLive() {
        timeOffsetMs.value = -driftMs.value;
    }

    /**
     * Put the view at an arbitrary moment, forwards included.
     *
     * Only the developer jump uses this: for everyone else a moment that has not
     * happened yet is not reachable, which is what [`setRewind`] enforces.
     */
    function seek(ms: number) {
        timeOffsetMs.value = ms - virtualElapsed.value;
    }

    function reset() {
        virtualElapsed.value = Date.now() - START_MS;
        timeOffsetMs.value = 0;
        driftMs.value = 0;
        timeSpeed.value = 1;
    }

    return {
        elapsedMs,
        tps,
        timeSpeed,
        timeOffsetMs,
        liveMs,
        maxRewindMs,
        rewindMs,
        aheadMs,
        atLive,
        setRewind,
        goLive,
        seek,
        reset,
    };
}

/**
 * Absolute time for display, down to the second.
 *
 * `Date` only covers about ±275,000 years, and the later milestones are far
 * past that — asking it for a date there yields `Invalid Date`, so out-of-range
 * moments are described as a distance from the start instead.
 */
export function formatMoment(ms: number, lang: Lang = 'en'): string {
    if (!Number.isFinite(ms)) return translate(lang, 'never');
    if (Math.abs(ms) > 8.64e15) return `${formatDuration(ms, lang)} ${translate(lang, 'after start')}`;
    const date = new Date(ms);
    if (Number.isNaN(date.getTime())) {
        return `${formatDuration(ms, lang)} ${translate(lang, 'after start')}`;
    }
    return date.toLocaleString(lang === 'zh' ? 'zh-CN' : 'en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
    });
}

/** Format milliseconds as `1 year 2 days 3 hours ...` (Chinese has no plurals). */
export function formatDuration(ms: number, lang: Lang = 'en'): string {
    if (ms <= 0) return translate(lang, '0 seconds');
    let s = ms / 1000;
    const years = Math.floor(s / (86400 * 365));
    s %= 86400 * 365;
    const days = Math.floor(s / 86400);
    s %= 86400;
    const hours = Math.floor(s / 3600);
    s %= 3600;
    const minutes = Math.floor(s / 60);
    s %= 60;
    const parts: string[] = [];
    // English takes the singular and adds `s`; Chinese looks the plural up.
    const push = (v: number, singular: string, zhKey: string) => {
        if (v <= 0) return;
        parts.push(
            lang === 'zh' ? `${v} ${translate(lang, zhKey)}` : `${v} ${singular}${v > 1 ? 's' : ''}`,
        );
    };
    push(years, 'year', 'years');
    push(days, 'day', 'days');
    push(hours, 'hour', 'hours');
    push(minutes, 'minute', 'minutes');
    if (s > 0) {
        parts.push(
            lang === 'zh' ? `${s.toFixed(2)} ${translate(lang, 'seconds')}` : `${s.toFixed(2)} seconds`,
        );
    }
    return parts.length ? parts.join(' ') : translate(lang, '0 seconds');
}
