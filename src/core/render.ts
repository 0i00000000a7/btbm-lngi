import katex from 'katex';
import {
    bocfBounds,
    bocfLatex,
    dbms,
    oneY,
    renderOrdinal,
    veblenLatex,
    vulcanizText,
    zeroY,
    type DisplayMode,
} from './wasm';

/**
 * Turning an ordinal into markup.
 *
 * Kept apart from the app so a panel can render itself: a panel that is
 * scrolled out of view should not be rendered at all, and only the component
 * that owns the element knows that.
 *
 * The converted notations are typeset by KaTeX. Each one is a rewrite of the
 * BTBMS term into a weaker system, so each has a ceiling: Veblen stops at the
 * Bachmann–Howard ordinal, BOCF at the collapse limit. A term past a notation's
 * reach is shown as ordinary BTBMS — unless it sits between two pinned points,
 * in which case it is printed between the two BOCF terms that bound it, with
 * `X` standing in for the ordinal.
 *
 * The pinned table is consulted *before* the conversion, not after it. The two
 * ranges are disjoint — the table begins where the converter stops — so the
 * order cannot change the answer, and the table is a binary search while the
 * conversion costs milliseconds on a long ordinal only to say "not
 * convertible". Past the limit, which is where the walk spends almost all its
 * time, that turns the fallback into the cheap path.
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

function renderConverted(plain: string, mode: DisplayMode, cols: number, value: number): string {
    const hit = mathMemo.get(mode);
    if (hit && hit.plain === plain) return hit.html;

    const pinned = pinnedReading(plain, value);
    if (pinned !== null) {
        mathMemo.set(mode, { plain, html: pinned });
        return pinned;
    }

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
 * KaTeX output for the hand-written bounds, keyed by the LaTeX itself.
 *
 * Unlike a converted term, a bound is one of a handful of constant strings, so
 * this cache stays small; without it every frame past the converter's reach
 * would typeset the same two symbols again.
 */
const boundHtml = new Map<string, string>();

function katexBound(latex: string): string {
    let html = boundHtml.get(latex);
    if (html === undefined) {
        html = katex.renderToString(latex, {
            output: 'html',
            throwOnError: false,
            strict: false,
        });
        boundHtml.set(latex, html);
    }
    return html;
}

/**
 * Past the converter's reach a few ordinals are pinned by hand. One that falls
 * between two of them is shown as `lower < X < upper`: the exact term is
 * unknown, but its position is not. The matrix itself would be far wider than
 * the two bounds around it, so `X` stands in for it — boxed, which is the usual
 * mark for a value that is known only to be in this range.
 *
 * `null` means the ordinal is not inside the table at all.
 */
function pinnedReading(plain: string, value: number): string | null {
    const bounds = bocfBounds(value, plain);
    if (!bounds) return null;
    return `<span class="pinned-reading">${katexBound(`{${bounds[0]}} < \\boxed{X} < {${bounds[1]}}`)}</span>`;
}

/**
 * Before the start the walk sits on the empty matrix. The OCF forms print the
 * zero; the matrix-shaped notations show the empty-set sign instead, which is
 * what an empty matrix is — and is not a dash, which would read as "unknown".
 */
export const EMPTY_HTML = '<span class="empty-ordinal">∅</span>';
export const ZERO_HTML = katex.renderToString('0', {
    output: 'html',
    throwOnError: false,
    strict: false,
});
export const ZERO_TEXT = '0';

/** What a box shows before the walk starts, which depends only on the mode. */
export function idleHtml(mode: DisplayMode): string {
    return mode === 'bocf' || mode === 'veblen' ? ZERO_HTML : EMPTY_HTML;
}

/**
 * One slot per mode, holding the arguments it was rendered for.
 *
 * Two boxes on the same mode render the same string, and a paused view renders
 * the same string every frame; both come out of this cache instead of the core.
 */
const memo = new Map<DisplayMode, { plain: string; cols: number; html: string }>();

/**
 * Every mode is a rewrite of the same term, and every rewrite has a ceiling.
 * Past it the BTBMS form is shown instead of a wrong answer.
 */
export function renderMode(
    mode: DisplayMode,
    plain: string,
    cols: number,
    value: number,
): string {
    const hit = memo.get(mode);
    if (hit && hit.plain === plain && hit.cols === cols) return hit.html;

    const html = renderFresh(mode, plain, cols, value);
    memo.set(mode, { plain, cols, html });
    return html;
}

function renderFresh(mode: DisplayMode, plain: string, cols: number, value: number): string {
    switch (mode) {
        case 'bocf':
        case 'veblen':
            return renderConverted(plain, mode, cols, value);
        case 'zeroy':
            return zeroY(plain) ?? renderOrdinal(plain, 'index', false, cols);
        case 'dbms':
            return dbms(plain) ?? renderOrdinal(plain, 'index', false, cols);
        case 'y1':
            return oneY(plain) ?? renderOrdinal(plain, 'index', false, cols);
        case 'vulcaniz':
            return vulcanizText(plain) ?? renderOrdinal(plain, 'index', false, cols);
        default:
            return renderOrdinal(plain, mode, false, cols);
    }
}
