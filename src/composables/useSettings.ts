import { ref, watch } from 'vue';
import { DISPLAY_MODES, type DisplayMode } from '../core/wasm';

/** The hero's mode setting: a fixed structure, or `auto` (see `App.vue`). */
export type HeroMode = DisplayMode | 'auto';

/** Which half of the milestone card is showing. */
export type View = 'progress' | 'list';

/** The three readings the hero caption cycles through. */
export type CaptionMode = 'ordinal' | 'milestone' | 'elapsed';

export interface Panel {
    id: string;
    mode: DisplayMode;
    width: number;
    /** Fixed height (px) of the panel body; draggable via the handle at the bottom. */
    height: number;
    /** Folded away: the header stays, the body does not render. */
    collapsed: boolean;
    hue: number;
    /** Magnification of this panel's text; 1 is its laid-out size. */
    scale: number;
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
    /** Magnification of the hero's text; 1 shows it at its laid-out size. */
    heroScale?: number;
    /**
     * Magnification that used to be shared by the hero and every panel.
     *
     * Only read to migrate: the hero keeps the value, the panels start at 1.
     */
    ordinalScale?: number;
    /** Precision levels of the rewind control, largest first. */
    rewindUnits: string[];
    /** Cards the user folded away, by id. */
    collapsedCards: string[];
    /** Which of the lower cards is on screen. */
    view: View;
    /** Show the time left to the next landmark instead of the time elapsed. */
    countdown?: boolean;
    /** Which reading the hero caption shows; supersedes `countdown`. */
    captionMode?: CaptionMode;
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
        // A panel saved before each box kept its own scale has none; it starts
        // at the laid-out size rather than inheriting the hero's magnification.
        scale: clampScale(p.scale ?? 1),
    };
}

/** Default cap on rendered columns (`0` = render every column). */
export const DEFAULT_MAX_COLUMNS = 0;

/** Default precision levels for the rewind control. */
export const DEFAULT_REWIND_UNITS = ['day', 'hour', 'minute'];

/**
 * Bounds of the ordinal magnification.
 *
 * The text is laid out at the box's own width and then magnified, so zooming
 * costs nothing to draw and only trades how much of the ordinal is on screen
 * for how large it is. There is no ceiling: the sub- and superscripts are
 * drawn at a fraction of the body size, so how far in a reader needs to go
 * depends on how deeply the column nests, and only they can tell where the
 * text stops being legible.
 *
 * The floor is 1: below that the layout would be shrunk into the corner of the
 * box, which shows less than the laid-out size does, not more.
 */
export const MIN_ORDINAL_SCALE = 1;

function clampScale(value: number): number {
    if (!Number.isFinite(value)) return 1;
    return Math.max(MIN_ORDINAL_SCALE, value);
}

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
    /** What the hero was magnified by before each box kept its own scale. */
    const legacyScale = clampScale(saved?.ordinalScale ?? 1);
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
                      scale: 1,
                  },
              ],
    );
    /** Height of the hero ordinal box; fixed and draggable like the panels. */
    const heroHeight = ref<number>(saved?.heroHeight ?? DEFAULT_PANEL_HEIGHT);
    /** Cap on rendered columns; every extra column multiplies the DOM cost. */
    const maxColumns = ref<number>(saved?.maxColumns ?? DEFAULT_MAX_COLUMNS);
    /** Magnification of the hero's text; 1 is its laid-out size. */
    const heroScale = ref<number>(clampScale(saved?.heroScale ?? legacyScale));
    /** Which granularities the rewind control offers. */
    const rewindUnits = ref<string[]>(saved?.rewindUnits ?? [...DEFAULT_REWIND_UNITS]);
    /** Folded cards, by id. */
    const collapsedCards = ref<string[]>(saved?.collapsedCards ?? []);
    /** The visible body of the milestone card. */
    const view = ref<View>(saved?.view === 'list' ? 'list' : 'progress');
    /** Which reading the hero caption shows: ordinal, milestone, or elapsed. */
    const captionMode = ref<CaptionMode>(
        saved?.captionMode ?? (saved?.countdown ? 'milestone' : 'elapsed'),
    );
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
            heroScale: heroScale.value,
            rewindUnits: rewindUnits.value,
            collapsedCards: collapsedCards.value,
            view: view.value,
            captionMode: captionMode.value,
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
            heroScale,
            rewindUnits,
            collapsedCards,
            view,
            captionMode,
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
            scale: 1,
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

    /**
     * Set the hero's magnification, clamped to the range the boxes stay usable
     * in. Each panel is magnified on its own: they show different notations at
     * different depths, so one shared factor never suits more than one of them.
     */
    function setHeroScale(value: number) {
        heroScale.value = clampScale(value);
    }

    /** Set one panel's magnification, clamped like the hero's. */
    function setPanelScale(id: string, value: number) {
        const panel = panels.value.find((p) => p.id === id);
        if (panel) panel.scale = clampScale(value);
    }

    return {
        fontFamily,
        fontSize,
        lang,
        defaultMode,
        panels,
        heroHeight,
        maxColumns,
        heroScale,
        rewindUnits,
        collapsedCards,
        view,
        captionMode,
        searchLayer,
        toggleCard,
        showSettings,
        addPanel,
        togglePanel,
        removePanel,
        setHeroScale,
        setPanelScale,
    };
}
