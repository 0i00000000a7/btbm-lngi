/**
 * Drawing an ordinal into a canvas.
 *
 * The DOM path costs a parse, a style pass and a full text layout on every
 * frame — about half of the main thread in a phone profile — and the result is
 * thrown away again by the next frame, since the ordinal changes every tick.
 * Most of that text is below the fold of a fixed-height box as well.
 *
 * Here the markup is split into runs once per frame, wrapped by hand using
 * cached character widths, and only the lines inside the visible part of the
 * box are drawn. The browser parses nothing and lays out nothing.
 *
 * Only the two tags the core emits — `<sup>` for a nested height and `<sub>`
 * for a column index, the latter sometimes wrapped in a coloured `<span>` —
 * are understood. Anything else, notably KaTeX output and the empty-set and
 * pinned-reading spans, makes [`parseRuns`] return `null`, and the caller
 * falls back to the DOM.
 */

/** A stretch of text sharing a script depth and a colour. */
export interface Run {
    text: string;
    /** Superscript depth: positive for `<sup>`, negative for `<sub>`. */
    script: number;
    color: string | null;
}

/** A run clipped to one line. */
export interface Segment {
    text: string;
    script: number;
    color: string | null;
    width: number;
}

export interface Line {
    segments: Segment[];
    width: number;
}

/** Size of each nested superscript, matching `sup { font-size: 0.62em }`. */
const SHRINK = 0.62;

/**
 * Smallest a nested script is drawn at, and smallest a level is lifted above
 * the one it sits in — in *screen* pixels.
 *
 * Every level shrinks by `SHRINK`, so by the sixth a script is under a pixel
 * and by the fortieth the size underflows to zero. A zero size is not a valid
 * font, so `ctx.font = …` is silently ignored and the measurement falls back
 * to whatever font happened to be left on the context — a full-size advance
 * for a character that draws nothing, which is what used to leave a blank
 * stretch in the middle of a deeply nested line.
 *
 * Being a screen measurement, the floor is divided by the magnification
 * before it is used: a box zoomed in lays the deep levels out proportionally
 * smaller, so magnifying shows the same shape at a larger size instead of
 * growing a staircase of full-size glyphs out of the nested end.
 */
const MIN_PX = 1;

const TAG = /<(\/?)([a-zA-Z][\w-]*)((?:"[^"]*"|'[^']*'|[^>"'])*)>/g;
const COLOR = /color:\s*(#[0-9a-fA-F]{3,8})/;

/**
 * Split rendered HTML into runs, or `null` if it holds markup this module does
 * not draw.
 *
 * Text is kept verbatim: the core emits no entities, so anything carrying one
 * is markup we do not understand either.
 */
export function parseRuns(html: string): Run[] | null {
    const runs: Run[] = [];
    const colors: (string | null)[] = [null];
    let script = 0;
    let at = 0;

    TAG.lastIndex = 0;
    for (let match = TAG.exec(html); match !== null; match = TAG.exec(html)) {
        const text = html.slice(at, match.index);
        if (text.includes('<') || text.includes('&')) return null;
        if (text !== '') runs.push({ text, script, color: colors[colors.length - 1] });

        const closing = match[1] === '/';
        const name = match[2].toLowerCase();
        if (name === 'sup') {
            script += closing ? -1 : 1;
        } else if (name === 'sub') {
            script -= closing ? -1 : 1;
        } else if (name === 'span') {
            if (closing) {
                if (colors.length === 1) return null;
                colors.pop();
            } else {
                const color = COLOR.exec(match[3]);
                if (color === null) return null;
                colors.push(color[1]);
            }
        } else {
            return null;
        }
        at = TAG.lastIndex;
    }

    const tail = html.slice(at);
    if (tail.includes('<') || tail.includes('&')) return null;
    if (tail !== '') runs.push({ text: tail, script, color: colors[colors.length - 1] });
    if (script !== 0 || colors.length !== 1) return null;
    return runs;
}

/**
 * Wraps runs into lines and draws the visible ones.
 *
 * One painter per canvas. It owns the character-width cache and the last
 * wrapping, both of which are invalidated when the typography changes.
 */
export class OrdinalPainter {
    private readonly ctx: CanvasRenderingContext2D;
    private readonly widths = new Map<string, number>();

    private family = '';
    private weight = '400';
    private size = 15;
    private lineHeight = 24;
    private color = '#000';

    private wrappedKey = '';
    private wrapped: Line[] = [];
    /** `MIN_PX` in layout pixels: the screen floor divided by the magnification. */
    private floor = MIN_PX;

    constructor(ctx: CanvasRenderingContext2D) {
        this.ctx = ctx;
    }

    /** Height of one line, in CSS pixels. */
    get lineHeightPx(): number {
        return this.lineHeight;
    }

    /**
     * Take the typography from the element the canvas fills.
     *
     * The app sets the font on an ancestor and the boxes differ in size, so the
     * only reliable source is the computed style. Reading it is cheap while
     * nothing invalidates it, and it notices a font change on its own.
     */
    syncStyle(el: Element): void {
        const cs = getComputedStyle(el);
        const size = parseFloat(cs.fontSize) || 15;
        const lineHeight =
            cs.lineHeight === 'normal' ? size * 1.5 : parseFloat(cs.lineHeight) || size * 1.5;
        if (
            size === this.size &&
            lineHeight === this.lineHeight &&
            cs.fontFamily === this.family &&
            cs.fontWeight === this.weight &&
            cs.color === this.color
        ) {
            return;
        }
        this.size = size;
        this.lineHeight = lineHeight;
        this.family = cs.fontFamily;
        this.weight = cs.fontWeight;
        this.color = cs.color;
        this.widths.clear();
        this.wrappedKey = '';
    }

    /**
     * Wrap `runs` to `width`, reusing the previous result when nothing moved.
     *
     * `scale` is the magnification the result will be drawn at. The nested
     * floors are screen measurements, so they are divided by it here: zoomed
     * in, the deep levels are laid out smaller and the shape stays the same.
     */
    layout(html: string, runs: Run[], width: number, scale: number): Line[] {
        // Quantised to quarter steps so a pinch does not re-wrap on every
        // frame; the deepest levels are a smudge either way, and this keeps
        // the cost of a gesture to one layout per step.
        const floor = MIN_PX / Math.pow(1.25, Math.round(Math.log(scale) / Math.log(1.25)));
        if (floor !== this.floor) {
            this.floor = floor;
            // The advances were measured against the old floor.
            this.widths.clear();
            this.wrappedKey = '';
        }
        const key = `${width}\u0000${html}`;
        if (key === this.wrappedKey) return this.wrapped;
        this.wrapped = this.wrap(runs, width);
        this.wrappedKey = key;
        return this.wrapped;
    }

    /**
     * Draw the lines that intersect the visible window.
     *
     * `scale` magnifies the drawn text without re-wrapping it: `lines` is
     * whatever width the caller chose to wrap at, so scaling up shows fewer
     * characters and pans, exactly as magnifying an image does, while scaling
     * down (with the caller having wrapped wider) fits more of the text in.
     * The canvas keeps its size, so the cost of drawing does not grow with the
     * zoom — only the glyphs do.
     */
    paint(
        lines: Line[],
        scrollLeft: number,
        scrollTop: number,
        width: number,
        height: number,
        dpr: number,
        scale: number,
    ): void {
        const ctx = this.ctx;
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.clearRect(0, 0, width * dpr, height * dpr);
        ctx.setTransform(
            dpr * scale,
            0,
            0,
            dpr * scale,
            -scrollLeft * dpr,
            -scrollTop * dpr,
        );
        ctx.textBaseline = 'middle';

        const lh = this.lineHeight;
        // The scroll offset is in magnified pixels; the lines are not.
        const top = scrollTop / scale;
        const first = Math.max(0, Math.floor(top / lh));
        const last = Math.min(lines.length, Math.ceil((top + height / scale) / lh));
        for (let i = first; i < last; i++) {
            const y = i * lh + lh / 2;
            let x = 0;
            for (const segment of lines[i].segments) {
                ctx.font = this.font(segment.script);
                ctx.fillStyle = segment.color ?? this.color;
                ctx.fillText(segment.text, x, y + this.rise(segment.script));
                x += segment.width;
            }
        }
    }

    /**
     * Break anywhere, as `overflow-wrap: anywhere` does. The ordinal has no
     * spaces to break at, so this matches what the DOM produced.
     */
    private wrap(runs: Run[], maxWidth: number): Line[] {
        const lines: Line[] = [];
        let line: Line = { segments: [], width: 0 };

        for (const run of runs) {
            for (const ch of run.text) {
                const width = this.advance(ch, run.script);
                if (line.width + width > maxWidth && line.segments.length > 0) {
                    lines.push(line);
                    line = { segments: [], width: 0 };
                }
                const last = line.segments[line.segments.length - 1];
                if (last !== undefined && last.script === run.script && last.color === run.color) {
                    last.text += ch;
                    last.width += width;
                } else {
                    line.segments.push({ text: ch, script: run.script, color: run.color, width });
                }
                line.width += width;
            }
        }
        if (line.segments.length > 0) lines.push(line);
        return lines;
    }

    private advance(ch: string, script: number): number {
        const key = `${script}:${ch}`;
        let width = this.widths.get(key);
        if (width === undefined) {
            this.ctx.font = this.font(script);
            width = this.ctx.measureText(ch).width;
            this.widths.set(key, width);
        }
        return width;
    }

    private font(script: number): string {
        const scale = Math.pow(SHRINK, Math.abs(script));
        return `${this.weight} ${Math.max(this.floor, this.size * scale)}px ${this.family}`;
    }

    /**
     * Vertical offset of a script run from the middle of its line.
     *
     * `textBaseline` is `middle`, so a smaller font is already centred on the
     * line; this shifts it by roughly what `vertical-align: super`/`sub` does
     * to a baseline, allowing for the fact that the middle of a 0.62em run is
     * higher than the middle of a full-size one.
     *
     * Each level lifts by a share of the size it is drawn at, and the share
     * bottoms out with the sizes, so the deepest levels are each lifted by the
     * floor instead of by a fraction of a pixel.
     */
    private rise(script: number): number {
        if (script === 0) return 0;
        const n = Math.abs(script);
        const share = script > 0 ? 0.25 : 0.3;
        // The levels above `stepped` still shrink, so their lifts are a
        // geometric series; every level past it is at the floor.
        const stepped = Math.max(
            0,
            Math.min(n, Math.ceil(Math.log(this.floor / (share * this.size)) / Math.log(SHRINK))),
        );
        const shrink = ((share * this.size) * (1 - Math.pow(SHRINK, stepped))) / (1 - SHRINK);
        const total = shrink + (n - stepped) * this.floor;
        return script > 0 ? -total : total;
    }
}
