/**
 * Writing rendered ordinals into the page.
 *
 * This is the fallback path: an ordinal is normally painted into a canvas (see
 * `core/ordinal-canvas`), which keeps the HTML parser and the layout engine out
 * of the per-frame work. What lands here is markup the painter does not draw —
 * KaTeX output, the empty-set sign, a pinned reading — and those are short, so
 * they are written only when the string actually changes.
 *
 * Two things make even that cheap:
 *
 * - A rendered ordinal without markup is written as a text node, which skips
 *   the parser entirely.
 * - A string identical to the previous one is not written at all. That is what
 *   makes a paused or rewound view free.
 */
const lastWritten = new WeakMap<Element, string>();

/** Write `html` into `el`, doing as little work as the content allows. */
export function writeOrdinal(el: Element, html: string): void {
    if (lastWritten.get(el) === html) return;
    lastWritten.set(el, html);

    // `&` may start an entity, so anything carrying one still needs the parser.
    if (html.includes('<') || html.includes('&')) {
        el.innerHTML = html;
    } else {
        el.textContent = html;
    }
}
