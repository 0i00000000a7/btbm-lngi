import { ref, watch } from 'vue';
import { DISPLAY_MODES, type DisplayMode } from '../core/wasm';

export interface Panel {
    id: string;
    mode: DisplayMode;
    width: number;
    /** Fixed height (px) of the panel body; draggable via the handle at the bottom. */
    height: number;
    hue: number;
}

const STORAGE_KEY = 'btbm_lngi_settings_v2';

/** Default height (px) for a new panel. */
export const DEFAULT_PANEL_HEIGHT = 150;

/** Lower bound (px) when dragging the height. */
export const MIN_PANEL_HEIGHT = 60;

interface Persisted {
    fontFamily: string;
    fontSize: number;
    defaultMode: DisplayMode;
    panels: Panel[];
    /** Height (px) of the hero ordinal box. */
    heroHeight: number;
}

function load(): Persisted | null {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        return raw ? (JSON.parse(raw) as Persisted) : null;
    } catch {
        return null;
    }
}

const DEFAULT_FONT =
    "'JetBrains Mono', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";

/** Old default font; if loaded, treat it as unset (otherwise the user never sees the new default). */
const OLD_DEFAULT_FONT = 'ui-monospace, monospace';

/** Old mode name -> new mode name (`html` -> `Index`, `marked` -> `annotated`). */
const RENAMED_MODES: Record<string, DisplayMode> = {
    html: 'index',
    plain: 'index',
    marked: 'annotated',
};

/** Migrate old mode names in persisted data to the current ones. */
function normalizeMode(mode: string | undefined, fallback: DisplayMode): DisplayMode {
    if (!mode) return fallback;
    const renamed = RENAMED_MODES[mode];
    if (renamed) return renamed;
    return DISPLAY_MODES.some((m) => m.id === mode) ? (mode as DisplayMode) : fallback;
}

/** Fill in fields missing from old data (e.g. the newer `height`). */
function normalizePanel(p: Panel, fallbackMode: DisplayMode): Panel {
    return { ...p, mode: normalizeMode(p.mode, fallbackMode), height: p.height ?? DEFAULT_PANEL_HEIGHT };
}

/** Settings + ordinal-panel layout (persisted in localStorage). */
export function useSettings() {
    const saved = load();

    const savedFont = saved?.fontFamily && saved.fontFamily !== OLD_DEFAULT_FONT ? saved.fontFamily : null;
    const fontFamily = ref(savedFont ?? DEFAULT_FONT);
    const fontSize = ref(saved?.fontSize ?? 16);
    const defaultMode = ref<DisplayMode>(normalizeMode(saved?.defaultMode, 'layer'));
    const panels = ref<Panel[]>(
        saved?.panels?.length
            ? saved.panels.map((p) => normalizePanel(p, defaultMode.value))
            : [
                  {
                      id: 'p0',
                      mode: defaultMode.value,
                      width: 100,
                      height: DEFAULT_PANEL_HEIGHT,
                      hue: 210,
                  },
              ],
    );
    /** Height of the hero ordinal box; fixed and draggable like the panels. */
    const heroHeight = ref<number>(saved?.heroHeight ?? DEFAULT_PANEL_HEIGHT);
    const showSettings = ref(false);

    function persist() {
        const data: Persisted = {
            fontFamily: fontFamily.value,
            fontSize: fontSize.value,
            defaultMode: defaultMode.value,
            panels: panels.value,
            heroHeight: heroHeight.value,
        };
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
        } catch {
            /* ignore quota errors */
        }
    }
    watch([fontFamily, fontSize, defaultMode, panels, heroHeight], persist, { deep: true });

    function addPanel() {
        panels.value.push({
            id: 'p' + Math.random().toString(36).slice(2, 9),
            mode: defaultMode.value,
            width: 100,
            height: DEFAULT_PANEL_HEIGHT,
            hue: Math.floor(Math.random() * 360),
        });
    }

    function removePanel(id: string) {
        panels.value = panels.value.filter((p) => p.id !== id);
    }

    return { fontFamily, fontSize, defaultMode, panels, heroHeight, showSettings, addPanel, removePanel };
}
