/**
 * Structural comparison of plain-form ordinals for the developer tree.
 *
 * This mirrors `compare` in `src/rust/compare.rs` exactly: expressions are
 * lexicographic sequences of columns, columns lexicographic sequences of
 * entries, and an entry compares by value and then by its nested height. A
 * longer sequence is greater. Keeping it here (rather than adding a wasm
 * export) means the tree can enforce its order with no rebuild.
 */

/** The synthetic root of the tree: greater than every real ordinal. */
export const LIMIT = 'LIMIT';

interface Entry {
    value: number;
    height: Expr;
}

type Expr = Entry[][];

const cache = new Map<string, Expr>();

/** Parse the plain reading `(0)(1^(2))(2,1)` into nested columns of entries. */
function parse(input: string): Expr {
    let pos = 0;

    const skipSpaces = () => {
        while (input[pos] === ' ') pos++;
    };

    const number = (): number => {
        skipSpaces();
        const start = pos;
        while (pos < input.length && input[pos] >= '0' && input[pos] <= '9') pos++;
        // The display is 1-based: a displayed `0` is the internal -1 and is
        // dropped as a trailing filler, just like `from_display` does.
        return Number(input.slice(start, pos)) - 1;
    };

    const entry = (): Entry => {
        const value = number();
        skipSpaces();
        if (input[pos] === '^') {
            pos++;
            return { value, height: expr() };
        }
        return { value, height: [] };
    };

    const column = (): Entry[] => {
        pos++; // '('
        skipSpaces();
        const entries: Entry[] = [];
        if (input[pos] !== ')' && input[pos] !== ':') {
            entries.push(entry());
            skipSpaces();
            while (input[pos] === ',') {
                pos++;
                skipSpaces();
                if (input[pos] === ')') break;
                entries.push(entry());
                skipSpaces();
            }
        }
        skipSpaces();
        // Marked readings carry a `:index` suffix; tolerate them.
        if (input[pos] === ':') {
            pos++;
            while (pos < input.length && input[pos] >= '0' && input[pos] <= '9') pos++;
        }
        skipSpaces();
        if (input[pos] === ')') pos++;
        while (entries.length && entries[entries.length - 1].value === -1) entries.pop();
        return entries;
    };

    const expr = (): Expr => {
        const columns: Expr = [];
        skipSpaces();
        while (input[pos] === '(') {
            columns.push(column());
            skipSpaces();
        }
        return columns;
    };

    return expr();
}

function parsed(text: string): Expr {
    let result = cache.get(text);
    if (!result) {
        result = parse(text);
        cache.set(text, result);
    }
    return result;
}

function lex<T>(a: T[], b: T[], cmp: (x: T, y: T) => number): number {
    const shared = Math.min(a.length, b.length);
    for (let i = 0; i < shared; i++) {
        const ord = cmp(a[i], b[i]);
        if (ord !== 0) return ord;
    }
    // `longer_last`: the longer sequence is greater.
    return Math.sign(a.length - b.length);
}

const entryCmp = (a: Entry, b: Entry): number =>
    Math.sign(a.value - b.value) || exprCmp(a.height, b.height);

const columnCmp = (a: Entry[], b: Entry[]): number => lex(a, b, entryCmp);

const exprCmp = (a: Expr, b: Expr): number => lex(a, b, columnCmp);

/** Negative when `a < b`, positive when `a > b`, zero when equal. */
export function compareOrdinal(a: string, b: string): number {
    if (a === LIMIT) return b === LIMIT ? 0 : 1;
    if (b === LIMIT) return -1;
    return exprCmp(parsed(a), parsed(b));
}
