import { ref, watch } from 'vue';
import { DISPLAY_MODES, type DisplayMode } from '../core/wasm';

/** The hero's mode setting: a fixed structure, or `auto` (see `App.vue`). */
export type HeroMode = DisplayMode | 'auto';

/** Which half of the milestone card is showing. */
export type View = 'progress' | 'list';

export interface Panel {
    id: string;
    mode: DisplayMode;
    width: number;
    /** Fixed height (px) of the panel body; draggable via the handle at the bottom. */
    height: number;
    /** Folded away: the header stays, the body does not render. */
    collapsed: boolean;
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
    /** UI language: only the mode names are translated so far. */
    lang: 'en' | 'zh';
    defaultMode: HeroMode;
    panels: Panel[];
    /** Height (px) of the hero ordinal box. */
    heroHeight: number;
    /** Show at most this many columns, then an ellipsis (0 = no limit). */
    maxColumns: number;
    /** Precision levels of the rewind control, largest first. */
    rewindUnits: string[];
    /** Cards the user folded away, by id. */
    collapsedCards: string[];
    /** Which of the lower cards is on screen. */
    view: View;
    /** Show the time left to the next landmark instead of the time elapsed. */
    countdown: boolean;
    /** Read the search field by column level rather than by parent column. */
    searchLayer: boolean;
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
    return {
        ...p,
        mode: normalizeMode(p.mode, fallbackMode),
        height: p.height ?? DEFAULT_PANEL_HEIGHT,
        collapsed: p.collapsed ?? false,
    };
}

/** Default cap on rendered columns (`0` = render every column). */
export const DEFAULT_MAX_COLUMNS = 0;

/** Default precision levels for the rewind control. */
export const DEFAULT_REWIND_UNITS = ['day', 'hour', 'minute'];

/**
 * Settings + ordinal-panel layout (persisted in localStorage).
 */
export function useSettings() {
    const saved = load();

    const savedFont = saved?.fontFamily && saved.fontFamily !== OLD_DEFAULT_FONT ? saved.fontFamily : null;
    const fontFamily = ref(savedFont ?? DEFAULT_FONT);
    const fontSize = ref(saved?.fontSize ?? 16);
    const lang = ref<'en' | 'zh'>(saved?.lang === 'zh' ? 'zh' : 'en');
    // `auto` is a hero-only choice; panels always carry a concrete mode.
    const defaultMode = ref<HeroMode>(
        saved?.defaultMode === 'auto' ? 'auto' : normalizeMode(saved?.defaultMode, 'layer'),
    );
    const panelMode = (): DisplayMode => (defaultMode.value === 'auto' ? 'layer' : defaultMode.value);
    const panels = ref<Panel[]>(
        saved?.panels?.length
            ? saved.panels.map((p) => normalizePanel(p, panelMode()))
            : [
                  {
                      id: 'p0',
                      mode: panelMode(),
                      width: 100,
                      height: DEFAULT_PANEL_HEIGHT,
                      collapsed: false,
                      hue: 210,
                  },
              ],
    );
    /** Height of the hero ordinal box; fixed and draggable like the panels. */
    const heroHeight = ref<number>(saved?.heroHeight ?? DEFAULT_PANEL_HEIGHT);
    /** Cap on rendered columns; every extra column multiplies the DOM cost. */
    const maxColumns = ref<number>(saved?.maxColumns ?? DEFAULT_MAX_COLUMNS);
    /** Which granularities the rewind control offers. */
    const rewindUnits = ref<string[]>(saved?.rewindUnits ?? [...DEFAULT_REWIND_UNITS]);
    /** Folded cards, by id. */
    const collapsedCards = ref<string[]>(saved?.collapsedCards ?? []);
    /** The visible body of the milestone card. */
    const view = ref<View>(saved?.view === 'list' ? 'list' : 'progress');
    /** Count down to the next landmark instead of counting up from the start. */
    const countdown = ref(saved?.countdown ?? false);
    /** The search field's reading: level (BMS spelling) or parent (the walk's). */
    const searchLayer = ref(saved?.searchLayer ?? true);
    const showSettings = ref(false);

    function persist() {
        const data: Persisted = {
            fontFamily: fontFamily.value,
            fontSize: fontSize.value,
            lang: lang.value,
            defaultMode: defaultMode.value,
            panels: panels.value,
            heroHeight: heroHeight.value,
            maxColumns: maxColumns.value,
            rewindUnits: rewindUnits.value,
            collapsedCards: collapsedCards.value,
            view: view.value,
            countdown: countdown.value,
            searchLayer: searchLayer.value,
        };
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
        } catch {
            /* ignore quota errors */
        }
    }
    watch(
        [
            fontFamily,
            fontSize,
            lang,
            defaultMode,
            panels,
            heroHeight,
            maxColumns,
            rewindUnits,
            collapsedCards,
            view,
            countdown,
            searchLayer,
        ],
        persist,
        { deep: true },
    );

    function addPanel(mode: DisplayMode = panelMode()) {
        panels.value.push({
            id: 'p' + Math.random().toString(36).slice(2, 9),
            mode,
            width: 100,
            height: DEFAULT_PANEL_HEIGHT,
            collapsed: false,
            hue: Math.floor(Math.random() * 360),
        });
    }

    /** Fold or unfold a whole card (milestones, rewind). */
    function toggleCard(id: string) {
        collapsedCards.value = collapsedCards.value.includes(id)
            ? collapsedCards.value.filter((c) => c !== id)
            : [...collapsedCards.value, id];
    }

    /** Fold or unfold a panel; the layout is persisted like everything else. */
    function togglePanel(id: string) {
        const panel = panels.value.find((p) => p.id === id);
        if (panel) panel.collapsed = !panel.collapsed;
    }

    function removePanel(id: string) {
        panels.value = panels.value.filter((p) => p.id !== id);
    }

    return {
        fontFamily,
        fontSize,
        lang,
        defaultMode,
        panels,
        heroHeight,
        maxColumns,
        rewindUnits,
        collapsedCards,
        view,
        countdown,
        searchLayer,
        toggleCard,
        showSettings,
        addPanel,
        togglePanel,
        removePanel,
    };
}
