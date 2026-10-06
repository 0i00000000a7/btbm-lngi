<script setup lang="ts">
import { computed } from 'vue';
import { DISPLAY_MODES, isHtmlMode, lngiState, type DisplayMode } from './core/wasm';
import { formatDuration, useClock } from './composables/useClock';
import { MIN_PANEL_HEIGHT, useSettings } from './composables/useSettings';

const clock = useClock();
const settings = useSettings();

/** `import.meta` is not usable in the template, so pull it out in the script. */
const isDev = import.meta.env.DEV;

const started = computed(() => clock.elapsedMs.value >= 0);

/** Computed once per display mode, reused by every panel. */
const statesByMode = computed(() => {
    const map = {} as Record<DisplayMode, ReturnType<typeof lngiState>>;
    for (const { id } of DISPLAY_MODES) map[id] = lngiState(clock.elapsedMs.value, id);
    return map;
});
const main = computed(() => statesByMode.value[settings.defaultMode.value]);
const mainIsHtml = computed(() => isHtmlMode(settings.defaultMode.value));

/** Before the start point the ordinal is pinned to (0); only show the remaining time here. */
const elapsedText = computed(() =>
    started.value
        ? formatDuration(clock.elapsedMs.value)
        : `${formatDuration(-clock.elapsedMs.value)} until start`,
);

const modeLabel = (id: DisplayMode) => DISPLAY_MODES.find((m) => m.id === id)?.label ?? id;

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
                <span class="tag">live</span>
            </div>
            <button class="ghost" @click="settings.showSettings.value = true">Settings</button>
        </header>

        <section class="hero">
            <div class="hero-label">
                <span>Current ordinal</span>
                <span>{{ modeLabel(settings.defaultMode.value) }}</span>
            </div>
            <div
                v-if="mainIsHtml"
                class="hero-ordinal"
                :style="{ height: settings.heroHeight.value + 'px' }"
                v-html="main.ordinal"
            />
            <div v-else class="hero-ordinal" :style="{ height: settings.heroHeight.value + 'px' }">
                {{ main.ordinal }}
            </div>
            <div class="resize-handle" title="Drag to resize" @pointerdown="startResize(heroBox, $event)" />

            <div class="progress">
                <div class="bar"><div class="fill" :style="{ width: main.percent * 100 + '%' }" /></div>
                <div class="caption">
                    <span><b>{{ (main.percent * 100).toFixed(3) }}%</b> to next ordinal</span>
                    <span>{{ elapsedText }}</span>
                </div>
            </div>
        </section>

        <section class="stats">
            <div class="stat">
                <div class="label">Running at</div>
                <div class="value">{{ clock.tps.value.toFixed(1) }} tps</div>
            </div>
        </section>

        <section>
            <div class="section-head">
                <h2>Panels</h2>
                <button class="ghost" @click="settings.addPanel">+ Add panel</button>
            </div>
            <div class="panels">
                <div
                    v-for="panel in settings.panels.value"
                    :key="panel.id"
                    class="panel"
                    :style="{ flexBasis: `calc(${panel.width}% - 12px)`, '--hue': panel.hue }"
                >
                    <div class="panel-head">
                        <select v-model="panel.mode">
                            <option v-for="m in DISPLAY_MODES" :key="m.id" :value="m.id">{{ m.label }}</option>
                        </select>
                        <button class="ghost" @click="settings.removePanel(panel.id)">Remove</button>
                    </div>
                    <div
                        v-if="isHtmlMode(panel.mode)"
                        class="panel-body"
                        :style="{ height: panel.height + 'px' }"
                        v-html="statesByMode[panel.mode].ordinal"
                    />
                    <div v-else class="panel-body" :style="{ height: panel.height + 'px' }">
                        {{ statesByMode[panel.mode].ordinal }}
                    </div>
                    <div
                        class="resize-handle"
                        title="Drag to resize"
                        @pointerdown="startResize(panel, $event)"
                    />
                </div>
            </div>
        </section>

        <div v-if="settings.showSettings.value" class="modal-backdrop" @click.self="settings.showSettings.value = false">
            <div class="modal">
                <h3>Settings</h3>
                <label>
                    Font family
                    <input v-model="settings.fontFamily.value" />
                </label>
                <label>
                    Font size
                    <input type="number" min="8" max="48" v-model.number="settings.fontSize.value" />
                </label>
                <label>
                    Default display mode
                    <select v-model="settings.defaultMode.value">
                        <option v-for="m in DISPLAY_MODES" :key="m.id" :value="m.id">{{ m.label }}</option>
                    </select>
                </label>

                <template v-if="isDev">
                    <hr />
                    <h4>Time controls (dev only)</h4>
                    <label>
                        timeSpeed
                        <input type="number" step="0.1" v-model.number="clock.timeSpeed.value" />
                    </label>
                    <label>
                        timeOffset (seconds)
                        <input
                            type="number"
                            step="1"
                            :value="clock.timeOffsetMs.value / 1000"
                            @input="clock.timeOffsetMs.value = Number(($event.target as HTMLInputElement).value) * 1000"
                        />
                    </label>
                    <button class="ghost" @click="clock.reset()">Reset time</button>
                </template>

                <button class="primary" @click="settings.showSettings.value = false">Close</button>
            </div>
        </div>
    </div>
</template>
