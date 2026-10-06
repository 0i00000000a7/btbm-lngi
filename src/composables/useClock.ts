import { onUnmounted, ref } from 'vue';
import { START_MS } from '../core/wasm';

/**
 * A clock synchronized with the wall clock.
 *
 * - On load, `virtualElapsed = Date.now() - START_MS` (may be negative -> not started);
 * - Afterwards, each frame does `virtualElapsed += dt * timeSpeed`;
 * - Externally, `elapsedMs = virtualElapsed + timeOffsetMs`.
 */
export function useClock() {
    const virtualElapsed = ref(Date.now() - START_MS);
    const timeSpeed = ref(1);
    const timeOffsetMs = ref(0);
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

    function reset() {
        virtualElapsed.value = Date.now() - START_MS;
        timeOffsetMs.value = 0;
        timeSpeed.value = 1;
    }

    return { elapsedMs, tps, timeSpeed, timeOffsetMs, reset };
}

/** Format milliseconds as `1 year 2 days 3 hours ...`. */
export function formatDuration(ms: number): string {
    if (ms <= 0) return '0 seconds';
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
    const push = (v: number, unit: string) => {
        if (v > 0) parts.push(`${v} ${unit}${v > 1 ? 's' : ''}`);
    };
    push(years, 'year');
    push(days, 'day');
    push(hours, 'hour');
    push(minutes, 'minute');
    if (s > 0) parts.push(`${s.toFixed(2)} seconds`);
    return parts.length ? parts.join(' ') : '0 seconds';
}
