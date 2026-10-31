//! Buchholz OCF rendering of the BMS fragment.
//!
//! A port of the reference JavaScript converter: a BMS matrix (three rows) is
//! mapped to an ordinal term built from `ψ`, `Ω` and `ω`, which is then
//! rendered either as ASCII (to diff against the reference) or as LaTeX (for
//! KaTeX).
//!
//! Only the fragment strictly below
//! `(0,0,0)(1,1,1)(2,1,1)(3,1,0)(2,0,0)` converts. Past that the existing OCF
//! definition no longer reaches, so the caller falls back to the BTBMS display.
//! The reference blows its stack on the limit matrix itself, which is why the
//! range test runs **before** any conversion — never as error handling around
//! it.
//!
//! Term representation: `[]` (zero) or `[a, b, c]`, with `Rc` sharing so that
//! `add`/`sub`/`exp` can build new terms without deep copies.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::notation::Btbm;
use super::types::{Column, Expr};

/// A term of the reference notation: `[]` (zero) or `[a, b, c]`.
#[derive(Clone)]
enum Term {
    Zero,
    Node(Rc<Triple>),
}

struct Triple {
    /// Stable identity for the render cache. A counter is used rather than the
    /// `Rc` address because freed nodes' addresses get reused, which would let
    /// one term's rendering be served for a different term.
    id: usize,
    a: Term,
    b: Term,
    c: Term,
}

fn next_id() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

fn node(a: Term, b: Term, c: Term) -> Term {
    Term::Node(Rc::new(Triple {
        id: next_id(),
        a,
        b,
        c,
    }))
}

/// `ONE = [[], [], []]`.
fn one() -> Term {
    node(Term::Zero, Term::Zero, Term::Zero)
}

impl Term {
    fn is_zero(&self) -> bool {
        matches!(self, Term::Zero)
    }

    /// `a[0], a[1], a[2]`; only valid on a node.
    fn parts(&self) -> (&Term, &Term, &Term) {
        match self {
            Term::Node(t) => (&t.a, &t.b, &t.c),
            Term::Zero => panic!("zero has no parts"),
        }
    }

    /// The node's stable identity; `0` for zero.
    fn id(&self) -> usize {
        match self {
            Term::Zero => 0,
            Term::Node(n) => n.id,
        }
    }

    /// `[a[0], a[1], []]`.
    fn head(&self) -> Term {
        let (a, b, _) = self.parts();
        node(a.clone(), b.clone(), Term::Zero)
    }
}

// ---------------------------------------------------------------------------
// Ordinal arithmetic on terms
// ---------------------------------------------------------------------------

fn eq(a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Zero, Term::Zero) => true,
        (Term::Node(x), Term::Node(y)) => {
            Rc::ptr_eq(x, y) || (eq(&x.a, &y.a) && eq(&x.b, &y.b) && eq(&x.c, &y.c))
        }
        _ => false,
    }
}

fn lt(a: &Term, b: &Term) -> bool {
    if b.is_zero() {
        return false;
    }
    if a.is_zero() {
        return true;
    }
    let (a0, a1, a2) = a.parts();
    let (b0, b1, b2) = b.parts();
    if !eq(a0, b0) {
        return lt(a0, b0);
    }
    if !eq(a1, b1) {
        return lt(a1, b1);
    }
    lt(a2, b2)
}

fn gt(a: &Term, b: &Term) -> bool {
    !(lt(a, b) || eq(a, b))
}

fn add(a: &Term, b: &Term) -> Term {
    if a.is_zero() {
        return b.clone();
    }
    if b.is_zero() {
        return a.clone();
    }
    if lt(&a.head(), &b.head()) {
        return b.clone();
    }
    let (a0, a1, a2) = a.parts();
    node(a0.clone(), a1.clone(), add(a2, b))
}

fn suc(a: &Term) -> Term {
    add(a, &one())
}

fn sub(a: &Term, b: &Term) -> Term {
    if a.is_zero() {
        return Term::Zero;
    }
    if b.is_zero() {
        return a.clone();
    }
    if gt(&a.head(), &b.head()) {
        return a.clone();
    }
    let (_, _, a2) = a.parts();
    let (_, _, b2) = b.parts();
    sub(a2, b2)
}

/// `s(a, b)`: split `a` at the first addend whose head is not below `b`.
fn split(a: &Term, b: &Term) -> (Term, Term) {
    if a.is_zero() {
        return (Term::Zero, Term::Zero);
    }
    if lt(&a.head(), b) {
        return (Term::Zero, a.clone());
    }
    let (a0, a1, a2) = a.parts();
    let (first, second) = split(a2, b);
    (node(a0.clone(), a1.clone(), first), second)
}

/// `l(a)`: the last addend.
fn last_addend(a: &Term) -> Term {
    if a.is_zero() {
        return Term::Zero;
    }
    let (_, _, a2) = a.parts();
    if a2.is_zero() {
        return a.clone();
    }
    last_addend(a2)
}

fn ttc(a: &Term, b: &Term) -> Term {
    if a.is_zero() {
        return Term::Zero;
    }
    let (a0, a1, a2) = a.parts();
    let tail = ttc(a2, b);
    if tail.is_zero() && lt(&a.head(), &node(b.clone(), Term::Zero, Term::Zero)) {
        return Term::Zero;
    }
    node(a0.clone(), a1.clone(), tail)
}

fn exp_term(a: &Term) -> Term {
    let threshold = node(
        Term::Zero,
        node(one(), Term::Zero, Term::Zero),
        Term::Zero,
    );
    if lt(a, &threshold) {
        return node(Term::Zero, a.clone(), Term::Zero);
    }
    let (a0, a1, _) = a.parts();
    let (p, _) = split(a1, &node(suc(a0), Term::Zero, Term::Zero));
    let inner = add(&p, &sub(a, &node(a0.clone(), p.clone(), Term::Zero)));
    node(a0.clone(), inner, Term::Zero)
}

fn log_term(a: &Term, c: &mut Caches) -> Term {
    if a.is_zero() {
        return Term::Zero;
    }
    let (a0, a1, a2) = a.parts();
    let (p, q) = split(a1, &node(suc(a0), Term::Zero, Term::Zero));

    if a0.is_zero() && p.is_zero() {
        if !lt(a1, &node(Term::Zero, node(one(), Term::Zero, Term::Zero), Term::Zero)) {
            if eq(&log_cached(&q, c), &q) && q.parts().2.is_zero() && lt(a1, &one()) {
                return node(a0.clone(), a1.clone(), Term::Zero);
            }
        }
        return q.clone();
    }

    let m = add(&node(a0.clone(), p.clone(), Term::Zero), &q);
    if !lt(
        a1,
        &node(
            a0.clone(),
            node(suc(a0), Term::Zero, Term::Zero),
            Term::Zero,
        ),
    ) && eq(&log_cached(a1, c), a1)
        && a2.is_zero()
        && lt(a1, &node(suc(a0), Term::Zero, Term::Zero))
    {
        return node(a0.clone(), a1.clone(), Term::Zero);
    }
    m
}

/// Memoized [`log_term`]. `log` is the expensive part of rendering, and the
/// same sub-terms recur constantly, so results are kept per node id.
fn log_cached(a: &Term, c: &mut Caches) -> Term {
    let key = a.id();
    if let Some(t) = c.log.get(&key) {
        return t.clone();
    }
    let r = log_term(a, c);
    c.log.insert(key, r.clone());
    r
}

/// `log([a0, a1, []])`, memoized on the two components so that the throwaway
/// head node does not defeat the cache.
fn log_of_head(a0: &Term, a1: &Term, c: &mut Caches) -> Term {
    let key = (a0.id(), a1.id());
    if let Some(t) = c.log_head.get(&key) {
        return t.clone();
    }
    let r = log_term(&node(a0.clone(), a1.clone(), Term::Zero), c);
    c.log_head.insert(key, r.clone());
    r
}

// ---------------------------------------------------------------------------
// BMS matrix helpers
// ---------------------------------------------------------------------------

/// A three-row matrix, as the reference expects it.
type Matrix = [[i64; 3]];

/// `P(M, r, n)`: the parent of row `r` of column `n`, or `-1`.
fn parent(m: &Matrix, r: i64, n: i64) -> i64 {
    if r == -1 {
        return n - 1;
    }
    let mut q = parent(m, r - 1, n);
    while q > -1 && m[q as usize][r as usize] >= m[n as usize][r as usize] {
        q = parent(m, r - 1, q);
    }
    q
}

/// `C(M, n)`: the columns whose row-0 parent is `n`.
fn children(m: &Matrix, n: i64) -> Vec<usize> {
    (0..m.len()).filter(|&i| parent(m, 0, i as i64) == n).collect()
}

/// `U(M, n)`: the column that supplies `n`'s predecessor, or `-1`.
fn uncle(m: &Matrix, n: i64) -> i64 {
    let len = m.len() as i64;
    if m[n as usize][1] == 0 || m[n as usize][2] == 1 || n + 1 == len {
        return -1;
    }
    let col = m[n as usize];
    // `m` in the reference: the row-1 parent, used both for its own sake and
    // for the target column it labels.
    let pm = parent(m, 1, n);
    if pm < 0 {
        return -1;
    }
    let target = [m[pm as usize][0] + 1, col[1], m[pm as usize][2] + 1];

    if pm == parent(m, 1, n + 1) && m[(n + 1) as usize] == target {
        return n + 1;
    }
    let mut q = n;
    loop {
        q = parent(m, 0, q);
        if q < 0 {
            return -1;
        }
        if pm == parent(m, 1, q)
            && m[q as usize] == target
            && m[(n + 1) as usize][0] > m[q as usize][0]
        {
            return q;
        }
    }
}

/// The recursion of `v`/`o` over one matrix.
///
/// The reference recomputes `C`, `U`, `v` and `o` from scratch on every visit,
/// which is exponential: near the top of the convertible range a single matrix
/// took a minute. Every one of them is a pure function of the column index, so
/// they are memoized per index here.
struct Converter {
    m: Vec<[i64; 3]>,
    children: Vec<Vec<usize>>,
    uncles: Vec<i64>,
    values: RefCell<Vec<Option<Term>>>,
    columns: RefCell<Vec<Option<Term>>>,
}

impl Converter {
    fn new(m: Vec<[i64; 3]>) -> Self {
        let children = (0..m.len())
            .map(|n| children(&m, n as i64))
            .collect();
        let uncles = (0..m.len()).map(|n| uncle(&m, n as i64)).collect();
        let len = m.len();
        Self {
            m,
            children,
            uncles,
            values: RefCell::new(vec![None; len]),
            columns: RefCell::new(vec![None; len]),
        }
    }

    /// `v(M, n)`: the ordinal of column `n`.
    fn value(&self, n: usize) -> Term {
        if let Some(t) = self.values.borrow()[n].clone() {
            return t;
        }
        let t = self.compute_value(n);
        self.values.borrow_mut()[n] = Some(t.clone());
        t
    }

    fn compute_value(&self, n: usize) -> Term {
        let col = self.m[n];
        if col[1] == 0 {
            return Term::Zero;
        }
        if col[2] == 0 {
            let u = self.uncles[n];
            let base = if u >= 0 {
                last_addend(&self.value(u as usize))
            } else {
                one()
            };
            let parent = parent(&self.m, 1, n as i64);
            return add(&self.value(parent.max(0) as usize), &base);
        }
        let mut p = one();
        for &i in &self.children[n] {
            if self.m[i] != [col[0] + 1, col[1], 1] {
                continue;
            }
            let mut q = Term::Zero;
            for &j in &self.children[i] {
                q = add(&q, &self.column(j));
            }
            p = add(&p, &exp_term(&q));
        }
        let parent = parent(&self.m, 1, n as i64);
        add(&self.value(parent.max(0) as usize), &exp_term(&p))
    }

    /// `o(M, n)`: column `n`'s ordinal together with its children's.
    fn column(&self, n: usize) -> Term {
        if let Some(t) = self.columns.borrow()[n].clone() {
            return t;
        }
        let t = self.compute_column(n);
        self.columns.borrow_mut()[n] = Some(t.clone());
        t
    }

    fn compute_column(&self, n: usize) -> Term {
        let col = self.m[n];
        let mut sum = Term::Zero;
        for &i in &self.children[n] {
            if self.m[i] == [col[0] + 1, col[1], 1] {
                continue;
            }
            if self.uncles.iter().any(|&u| u == i as i64) {
                match self.children[i].last() {
                    Some(&last) => {
                        if self.m[last] == [self.m[i][0] + 1, self.m[i][1], 1] {
                            continue;
                        }
                    }
                    None => continue,
                }
            }
            sum = add(&sum, &self.column(i));
        }
        node(self.value(n), sum, Term::Zero)
    }

    /// `NS(M)`: the additive sum over the root columns.
    fn ns(&self) -> Term {
        let mut sum = Term::Zero;
        for i in 0..self.m.len() {
            if self.m[i] == [0, 0, 0] {
                sum = add(&sum, &self.column(i));
            }
        }
        sum
    }
}

/// `sp(a, b, c)`: fold the addends of `c` into `b`, collapsing with `ttc`.
fn sp(a: &Term, b: &Term, c: &Term) -> Term {
    if c.is_zero() {
        return node(a.clone(), b.clone(), Term::Zero);
    }
    let (c0, c1, c2) = c.parts();
    if lt(b, c1) && gt(c, &node(a.clone(), Term::Zero, Term::Zero)) {
        let t = ttc(c1, &suc(c0));
        let next = add(
            &t,
            &sub(
                &node(c0.clone(), c1.clone(), Term::Zero),
                &node(c0.clone(), t.clone(), Term::Zero),
            ),
        );
        return sp(a, &next, c2);
    }
    sp(a, &add(b, &node(c0.clone(), c1.clone(), Term::Zero)), c2)
}

/// `sf(a)`: standard form.
fn sf(a: &Term) -> Term {
    if a.is_zero() {
        return Term::Zero;
    }
    let (a0, a1, a2) = a.parts();
    add(&sp(&sf(a0), &Term::Zero, &sf(a1)), &sf(a2))
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dialect {
    /// The reference's own output, used to diff against it.
    Ascii,
    /// What the UI shows.
    Latex,
}

/// `getCoef(x)`: the number of times the same addend repeats.
fn coefficient(x: &Term) -> u64 {
    if x.is_zero() {
        return 0;
    }
    let (_, _, c) = x.parts();
    if c.is_zero() {
        1
    } else {
        coefficient(c) + 1
    }
}

/// The value of a `[[], [], x]` successor chain, or `None` if `t` is not one.
fn numeral(t: &Term) -> Option<u64> {
    match t {
        Term::Zero => Some(0),
        Term::Node(n) if n.a.is_zero() && n.b.is_zero() => Some(1 + numeral(&n.c)?),
        _ => None,
    }
}

/// `length1`: how many addends the chain holds.
fn length1(a: &Term) -> u64 {
    match a {
        Term::Zero => 0,
        Term::Node(n) => 1 + length1(&n.c),
    }
}

/// A finite ordinal (a plain successor chain).
fn is_finite(a: &Term) -> bool {
    match a {
        Term::Zero => true,
        Term::Node(n) => n.a.is_zero() && n.b.is_zero(),
    }
}

/// `\omega`, `\Omega` or `\Omega_{a}` — the shape of the leading term.
fn omega_str(a: &Term, c: &mut Caches) -> String {
    if a.is_zero() {
        return "\\omega".to_string();
    }
    if eq(a, &one()) {
        return "\\Omega".to_string();
    }
    format!("\\Omega_{{{}}}", render_display(a, c))
}

/// Split a leading principal term into `(exponent, coefficient)`, i.e.
/// `a == \omega^exponent * coefficient`.
fn decompose_power(a: &Term, c: &mut Caches) -> (Term, Term) {
    if a.is_zero() {
        return (Term::Zero, Term::Zero);
    }
    let (a0, a1, _) = a.parts();
    if a0.is_zero() {
        return (log_cached(a, c), Term::Zero);
    }
    let (p, s) = split(a1, &node(suc(a0), Term::Zero, Term::Zero));
    let (q, r) = split(&s, &node(a0.clone(), Term::Zero, Term::Zero));
    let second = exp_term(&r);
    let mut first = add(&one(), &p);
    let mut ptr = q;
    while !ptr.is_zero() {
        let next = ptr.parts().2.clone();
        let log_val = log_cached(&ptr, c);
        let sub_val = sub(&log_val, &node(a0.clone(), Term::Zero, Term::Zero));
        first = add(&first, &exp_term(&sub_val));
        ptr = next;
    }
    (first, second)
}

/// The display form: Buchholz notation with `ψ`, `Ω` and `ω`.
///
/// Plain parentheses are used rather than `\left( \right)`: KaTeX scales
/// those to the tallest part of the group, which for a deep exponent makes the
/// brackets dominate the expression.
fn render_display(t: &Term, c: &mut Caches) -> String {
    if t.is_zero() {
        return "0".to_string();
    }
    if is_finite(t) {
        return length1(t).to_string();
    }
    let key = t.id();
    if let Some(s) = c.display.get(&key) {
        return s.clone();
    }

    let (a_part, b_part) = split(t, &t.head());
    let (a0, a1, _) = a_part.parts();

    let mut m = format!(
        "\\psi_{{{}}}({})",
        render_display(a0, c),
        render_display(a1, c)
    );
    if a1.is_zero() {
        m = omega_str(a0, c);
    }
    if a0.is_zero() {
        m = format!("\\psi({})", render_display(a1, c));
    }
    if a0.is_zero() && eq(a1, &one()) {
        m = "\\omega".to_string();
    } else if lt(a1, &node(suc(a0), Term::Zero, Term::Zero)) {
        let (first, second) = decompose_power(&a_part, c);
        if !eq(&first, &a_part) {
            m = omega_str(a0, c);
            if gt(&first, &one()) {
                m += &format!("^{{{}}}", render_display(&first, c));
            }
            if gt(&second, &one()) {
                m += &format!("\\cdot {}", render_display(&second, c));
            }
            let len = length1(&a_part);
            if len > 1 {
                m += &format!("\\cdot {len}");
            }
            if !b_part.is_zero() {
                m += &format!("+{}", render_display(&b_part, c));
            }
            c.display.insert(key, m.clone());
            return m;
        }
    }

    let len = length1(&a_part);
    if len > 1 {
        m += &format!("\\cdot {len}");
    }
    if !b_part.is_zero() {
        m += &format!("+{}", render_display(&b_part, c));
    }
    c.display.insert(key, m.clone());
    m
}

/// Per-render memo tables. Node ids are globally unique, so entries can never
/// be confused with a different term — which is exactly what a pointer-keyed
/// cache gets wrong once the allocator reuses an address.
#[derive(Default)]
struct Caches {
    render: HashMap<usize, String>,
    log: HashMap<usize, Term>,
    log_head: HashMap<(usize, usize), Term>,
    coef: HashMap<usize, u64>,
    display: HashMap<usize, String>,
}

fn render(t: &Term, d: Dialect) -> String {
    render_cached(t, d, &mut Caches::default())
}

fn coef_cached(a: &Term, c: &mut Caches) -> u64 {
    let key = a.id();
    if let Some(n) = c.coef.get(&key) {
        return *n;
    }
    let n = coefficient(a);
    c.coef.insert(key, n);
    n
}

fn render_cached(t: &Term, d: Dialect, c: &mut Caches) -> String {
    if t.is_zero() {
        return "0".to_string();
    }
    if let Some(n) = numeral(t) {
        return n.to_string();
    }
    let key = t.id();
    if let Some(s) = c.render.get(&key) {
        return s.clone();
    }

    let (a, b) = split(t, &t.head());
    let (a0, a1, _) = a.parts();

    let mut out = match d {
        Dialect::Ascii => format!("psi_{}({})", render_cached(a0, d, c), render_cached(a1, d, c)),
        Dialect::Latex => format!(
            "\\psi_{{{}}}({})",
            render_cached(a0, d, c),
            render_cached(a1, d, c)
        ),
    };

    if a1.is_zero() {
        out = match d {
            Dialect::Ascii => format!("Omega_{}", render_cached(a0, d, c)),
            Dialect::Latex => format!("\\Omega_{{{}}}", render_cached(a0, d, c)),
        };
    }
    if a1.is_zero() && eq(a0, &one()) {
        out = match d {
            Dialect::Ascii => "Omega".to_string(),
            Dialect::Latex => "\\Omega".to_string(),
        };
    }
    if a0.is_zero() {
        out = match d {
            Dialect::Ascii => format!("psi({})", render_cached(a1, d, c)),
            Dialect::Latex => format!("\\psi({})", render_cached(a1, d, c)),
        };
    }
    if a0.is_zero() && eq(a1, &one()) {
        out = match d {
            Dialect::Ascii => "w".to_string(),
            Dialect::Latex => "\\omega".to_string(),
        };
    } else {
        let lg = log_of_head(a0, a1, c);
        let head = node(a0.clone(), a1.clone(), Term::Zero);
        if !eq(&lg, &head) {
            out = match d {
                Dialect::Ascii => format!("w^{}", render_cached(&lg, d, c)),
                Dialect::Latex => format!("\\omega^{{{}}}", render_cached(&lg, d, c)),
            };
        }
    }

    let coef = coef_cached(&a, c);
    if coef > 1 {
        match d {
            Dialect::Ascii => out.push_str(&coef.to_string()),
            Dialect::Latex => out.push_str(&format!("\\cdot {coef}")),
        }
    }
    if !b.is_zero() {
        out.push('+');
        out.push_str(&render_cached(&b, d, c));
    }
    c.render.insert(key, out.clone());
    out
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// The largest matrix the OCF reaches; anything at or beyond it must fall back.
const LIMIT: &str = "(0,0,0)(1,1,1)(2,1,1)(3,1,0)(2,0,0)";

/// [`LIMIT`] parsed once — the parse is not free and this runs every frame.
fn limit_expr() -> &'static Expr {
    use std::sync::OnceLock;
    static PARSED: OnceLock<Expr> = OnceLock::new();
    PARSED.get_or_init(|| Btbm::new().from_display(LIMIT).expect("constant limit"))
}

/// How many columns are converted. Wider matrices are cut to this prefix and
/// marked with an ellipsis, exactly like the ordinary display's column cap.
///
/// The cost grows far faster than the matrix does — a 300-column matrix took a
/// minute and produced thousands of characters — while the result is already
/// unreadable long before that.
const MAX_COLUMNS: usize = 60;

/// The expression a `Column` stands for in the reference's matrix form.
///
/// The reference indexes rows directly and pads short columns with zeroes, so
/// a column with more than three entries, or one that still carries a nested
/// height, is outside what the converter can express.
fn matrix_column(col: &Column) -> Option<[i64; 3]> {
    let mut row = [0i64; 3];
    if col.len() > 3 {
        return None;
    }
    for (i, entry) in col.iter().enumerate() {
        if !entry.height.is_empty() {
            return None;
        }
        row[i] = entry.value + 1;
    }
    Some(row)
}

fn to_matrix(layered: &[Column]) -> Option<Vec<[i64; 3]>> {
    layered.iter().map(matrix_column).collect()
}

/// The Buchholz term for `expr`, or `None` when the ordinal is outside the
/// convertible range — which the callers turn into the ordinary BTBMS display.
fn term_of(n: &Btbm, expr: &Expr) -> Option<Term> {
    let layered = n.convert_to_layer(expr);
    if n.compare(&layered, limit_expr()) != std::cmp::Ordering::Less {
        return None;
    }
    // Overflow is cut off, not marked: the result is the prefix ordinal, shown
    // as-is with no ellipsis.
    let m = to_matrix(&layered[..layered.len().min(MAX_COLUMNS)])?;
    Some(sf(&Converter::new(m).ns()))
}

/// Render `expr` as a Buchholz term, in the given dialect.
fn convert(n: &Btbm, expr: &Expr, d: Dialect) -> Option<String> {
    let term = term_of(n, expr)?;
    Some(match d {
        Dialect::Ascii => render(&term, d),
        Dialect::Latex => render_display(&term, &mut Caches::default()),
    })
}

/// The ordinal as BOCF, in LaTeX; `None` when it is past what the OCF reaches.
pub fn bocf_latex(n: &Btbm, expr: &Expr) -> Option<String> {
    convert(n, expr, Dialect::Latex)
}

// ---------------------------------------------------------------------------
// Extended Veblen rendering (arXiv:2310.12832)
// ---------------------------------------------------------------------------
//
// `ψ₀(α)` is rewritten as `V(t(α))`, where `t` is the paper's t-function, and
// `render_array` prints the resulting Ω-normal form as a Veblen array. Only
// terms below the Bachmann–Howard ordinal have such a form, so `veblen_latex`
// can decline — which is what makes the BOCF fallback meaningful.

fn omega1() -> Term {
    node(one(), Term::Zero, Term::Zero)
}

fn omega() -> Term {
    node(Term::Zero, one(), Term::Zero)
}

/// `ψ₀(Ω₂)`, the Bachmann–Howard ordinal.
fn bho() -> Term {
    node(
        Term::Zero,
        node(suc(&one()), Term::Zero, Term::Zero),
        Term::Zero,
    )
}

fn from_int(n: u64) -> Term {
    let mut out = Term::Zero;
    for _ in 0..n {
        out = add(&out, &one());
    }
    out
}

/// The addends of a sum, each as a principal term.
fn every_terms(a: &Term) -> Vec<Term> {
    let mut out = Vec::new();
    let mut cur = a.clone();
    while !cur.is_zero() {
        out.push(cur.head());
        let next = cur.parts().2.clone();
        cur = next;
    }
    out
}

fn mul_finite(a: &Term, b: &Term) -> Term {
    if b.is_zero() {
        return Term::Zero;
    }
    let next = b.parts().2.clone();
    add(a, &mul_finite(a, &next))
}

/// `first_term`, tolerating zero (which has no parts).
fn first_term(a: &Term) -> Term {
    if a.is_zero() {
        Term::Zero
    } else {
        a.head()
    }
}

/// General ordinal multiplication.
fn mul(a: &Term, b: &Term, c: &mut Caches) -> Term {
    if a.is_zero() || b.is_zero() {
        return Term::Zero;
    }
    if lt(
        &first_term(&log_cached(a, c)),
        &first_term(&log_cached(b, c)),
    ) {
        return b.clone();
    }
    let (head, rest) = split(b, &omega());
    let log_a = log_cached(a, c);
    let mut result = mul_finite(a, &rest);
    let terms = every_terms(&head);
    for it in terms.iter().rev() {
        let term = exp_term(&add(&log_a, &log_cached(it, c)));
        let (t0, t1, _) = term.parts();
        result = node(t0.clone(), t1.clone(), result);
    }
    result
}

/// `k(α, β)` from the paper: `-1`, `0` or `1`.
fn k_function(alpha: &Term, beta: &Term, c: &mut Caches) -> i32 {
    if lt(alpha, &omega1()) {
        if lt(alpha, beta) {
            return -1;
        }
        return if eq(alpha, beta) { 0 } else { 1 };
    }

    // α ≥ Ω: split α into its Ω-power terms and a finite tail.
    let mut terms: Vec<(Term, Term)> = Vec::new();
    let mut tail = Term::Zero;
    let mut curr = alpha.clone();
    while !curr.is_zero() {
        let head = curr.head();
        if !head.parts().0.is_zero() {
            let (exp_t, second) = decompose_power(&head, c);
            terms.push((exp_t, second));
        } else {
            tail = add(&tail, &head);
        }
        let next = curr.parts().2.clone();
        curr = next;
    }

    let (gamma, delta) = if !tail.is_zero() {
        (Term::Zero, tail.clone())
    } else if !terms.is_empty() {
        (
            terms.last().unwrap().0.clone(),
            terms.last().unwrap().1.clone(),
        )
    } else {
        return if eq(alpha, beta) {
            0
        } else if lt(alpha, beta) {
            -1
        } else {
            1
        };
    };

    let mut all_minus_one = true;
    for (exp_t, coeff) in &terms {
        if k_function(exp_t, beta, c) != -1 || k_function(coeff, beta, c) != -1 {
            all_minus_one = false;
            break;
        }
    }
    if !tail.is_zero() && all_minus_one && k_function(&tail, beta, c) != -1 {
        all_minus_one = false;
    }
    if all_minus_one {
        return -1;
    }

    // The `k = 0` case looks only at the terms before the last group.
    let xi_end = if tail.is_zero() {
        terms.len().saturating_sub(1)
    } else {
        terms.len()
    };
    let xi_all_minus_one = terms[..xi_end]
        .iter()
        .all(|(e, k)| k_function(e, beta, c) == -1 && k_function(k, beta, c) == -1);
    if xi_all_minus_one
        && ((eq(&gamma, beta) && eq(&delta, &one()))
            || (k_function(&gamma, beta, c) == -1 && eq(&delta, beta)))
    {
        return 0;
    }

    1
}

/// `λ = ψ₀(ξ) − 1`.
fn psi0_minus_one(xi: &Term) -> Term {
    if xi.is_zero() {
        return Term::Zero;
    }
    node(Term::Zero, xi.clone(), Term::Zero)
}

struct OmegaCnfTerm {
    exponent: Term,
    second: Term,
    count: i32,
}

fn decompose_omega_cnf(
    alpha: &Term,
    terms: &mut Vec<OmegaCnfTerm>,
    tail: &mut Term,
    c: &mut Caches,
) {
    let mut curr = alpha.clone();
    while !curr.is_zero() {
        let head = curr.head();
        if !head.parts().0.is_zero() {
            let (exp_t, second) = decompose_power(&head, c);
            let merged = match terms.last_mut() {
                Some(last) if eq(&last.exponent, &exp_t) => {
                    let mut total = Term::Zero;
                    for _ in 0..last.count {
                        total = add(&total, &last.second);
                    }
                    last.second = add(&total, &second);
                    last.count = 1;
                    true
                }
                _ => false,
            };
            if !merged {
                terms.push(OmegaCnfTerm {
                    exponent: exp_t,
                    second,
                    count: 1,
                });
            }
        } else {
            *tail = add(tail, &head);
        }
        let next = curr.parts().2.clone();
        curr = next;
    }
}

/// The paper's `t(α)`.
fn compute_t(alpha: &Term, c: &mut Caches) -> Term {
    if alpha.is_zero() {
        return Term::Zero;
    }

    let mut terms: Vec<OmegaCnfTerm> = Vec::new();
    let mut tail = Term::Zero;
    decompose_omega_cnf(alpha, &mut terms, &mut tail, c);

    let (beta, gamma, xi) = if !tail.is_zero() {
        let mut xi = Term::Zero;
        let mut cur = alpha.clone();
        while !cur.is_zero() {
            let head = cur.head();
            if !head.parts().0.is_zero() {
                xi = add(&xi, &head);
            }
            let next = cur.parts().2.clone();
            cur = next;
        }
        (Term::Zero, tail.clone(), xi)
    } else if !terms.is_empty() {
        let beta = terms.last().unwrap().exponent.clone();
        let mut start = terms.len() as i32 - 1;
        while start >= 0 && eq(&terms[start as usize].exponent, &beta) {
            start -= 1;
        }
        start += 1;
        let mut gamma = Term::Zero;
        for tm in &terms[start as usize..] {
            for _ in 0..tm.count {
                gamma = add(&gamma, &tm.second);
            }
        }
        let mut xi = Term::Zero;
        let mut cur = alpha.clone();
        while !cur.is_zero() {
            let head = cur.head();
            if !head.parts().0.is_zero() {
                let (exp_t, _) = decompose_power(&head, c);
                if gt(&exp_t, &beta) {
                    xi = add(&xi, &head);
                }
            }
            let next = cur.parts().2.clone();
            cur = next;
        }
        (beta, gamma, xi)
    } else {
        return alpha.clone();
    };

    let lambda = psi0_minus_one(&xi);
    let rho = match k_function(&beta, &lambda, c) {
        -1 => lambda.clone(),
        0 => one(),
        _ => Term::Zero,
    };

    let sum = add(&rho, &gamma);
    let delta = if sum.is_zero() {
        Term::Zero
    } else if is_finite(&sum) {
        let n = length1(&sum);
        if n <= 1 {
            Term::Zero
        } else {
            from_int(n - 1)
        }
    } else {
        sum
    };

    add(&mul(&omega1(), &beta, c), &delta)
}

// ---------------------------------------------------------------------------
// Veblen rendering
// ---------------------------------------------------------------------------

fn has_omega_power_deep(t: &Term) -> bool {
    match t {
        Term::Zero => false,
        Term::Node(n) => {
            !n.a.is_zero() || has_omega_power_deep(&n.b) || has_omega_power_deep(&n.c)
        }
    }
}

/// `render_array` without the `\varphi`/`\omega^{...}` wrapper, for use as a
/// position inside another array.
fn render_array_body(alpha: &Term, is_position: bool, c: &mut Caches) -> String {
    if alpha.is_zero() {
        return "0".to_string();
    }
    let full = render_array(alpha, is_position, c);
    if full.len() > 8 && full.starts_with("\\varphi(") && full.ends_with(')') {
        return full[8..full.len() - 1].to_string();
    }
    if full.len() > 9 && full.starts_with("\\omega^{") && full.ends_with('}') {
        return full[8..full.len() - 1].to_string();
    }
    full
}

fn render_position(beta: &Term, c: &mut Caches) -> String {
    if beta.is_zero() {
        return "0".to_string();
    }
    if is_finite(beta) {
        return length1(beta).to_string();
    }
    if has_omega_power_deep(beta) {
        let body = render_array_body(beta, true, c);
        if body.contains(',') || body.contains('@') {
            if body.len() > 8 && body.starts_with("\\varphi(") && body.ends_with(')') {
                return body;
            }
            return format!("({body})");
        }
        return body;
    }
    render_display(beta, c)
}

fn render_veblen_coeff(t: &Term, c: &mut Caches) -> String {
    if t.is_zero() {
        return "0".to_string();
    }
    if is_finite(t) {
        return length1(t).to_string();
    }
    if has_omega_power_deep(t) {
        let v = render_veblen_rec(t, c);
        if !v.is_empty() {
            return v;
        }
    }
    render_display(t, c)
}

fn render_array(alpha: &Term, is_position: bool, c: &mut Caches) -> String {
    if alpha.is_zero() {
        return "1".to_string();
    }

    let mut terms: Vec<OmegaCnfTerm> = Vec::new();
    let mut tail = Term::Zero;
    decompose_omega_cnf(alpha, &mut terms, &mut tail, c);

    let has_complex_position = terms.iter().any(|tm| !is_finite(&tm.exponent));

    if !has_complex_position && terms.is_empty() {
        if tail.is_zero() {
            return "1".to_string();
        }
        {
            let (t0, _, t2) = tail.parts();
            if t2.is_zero() && t0.is_zero() && has_omega_power_deep(&tail) {
                let coeff = render_veblen_coeff(&tail, c);
                if coeff.len() >= 8 && coeff.starts_with("\\omega^{") {
                    return format!("\\omega^{{{coeff}}}");
                }
                return if is_position {
                    format!("\\varphi({coeff})")
                } else {
                    coeff
                };
            }
        }
        let coeff = render_veblen_coeff(&tail, c);
        if coeff == "1" {
            return "\\omega".to_string();
        }
        return format!("\\omega^{{{coeff}}}");
    }

    if !has_complex_position {
        let max_exp = terms.iter().map(|tm| length1(&tm.exponent)).max().unwrap_or(0);
        let mut coeff_terms: Vec<Term> = (0..=max_exp).map(|_| Term::Zero).collect();
        for tm in &terms {
            let pos = length1(&tm.exponent) as usize;
            let mut contrib = Term::Zero;
            if eq(&tm.second, &one()) || tm.second.is_zero() {
                for _ in 0..tm.count {
                    contrib = add(&contrib, &one());
                }
            } else {
                for _ in 0..tm.count {
                    contrib = add(&contrib, &tm.second);
                }
            }
            coeff_terms[pos] = add(&coeff_terms[pos], &contrib);
        }

        if !is_position {
            let tail_sugar = if tail.is_zero() {
                "0".to_string()
            } else {
                render_veblen_coeff(&tail, c)
            };
            if max_exp == 1 && eq(&coeff_terms[1], &one()) {
                return format!("\\varepsilon_{{{tail_sugar}}}");
            }
            if max_exp == 1 && eq(&coeff_terms[1], &add(&one(), &one())) {
                return format!("\\zeta_{{{tail_sugar}}}");
            }
            if max_exp == 1 && eq(&coeff_terms[1], &add(&add(&one(), &one()), &one())) {
                return format!("\\eta_{{{tail_sugar}}}");
            }
            if max_exp == 2 && eq(&coeff_terms[2], &one()) && coeff_terms[1].is_zero() {
                return format!("\\Gamma_{{{tail_sugar}}}");
            }
        }

        let mut result = "\\varphi(".to_string();
        let mut first = true;
        for i in (1..=max_exp).rev() {
            if !first {
                result += ",";
            }
            result += &render_veblen_coeff(&coeff_terms[i as usize], c);
            first = false;
        }
        if !first || !tail.is_zero() {
            if !first {
                result += ",";
            }
            if tail.is_zero() {
                result += "0";
            } else {
                result += &render_veblen_coeff(&tail, c);
            }
        }
        result += ")";
        return result;
    }

    // Positions that are not finite need the `c{@}p` array notation.
    let mut result = "\\varphi(".to_string();
    let mut first = true;
    for tm in &terms {
        if !first {
            result += ",";
        }
        let coeff = if eq(&tm.second, &one()) || tm.second.is_zero() {
            if tm.count == 1 {
                "1".to_string()
            } else {
                tm.count.to_string()
            }
        } else {
            let mut coeff_term = Term::Zero;
            for _ in 0..tm.count {
                coeff_term = add(&coeff_term, &tm.second);
            }
            render_veblen_coeff(&coeff_term, c)
        };
        result += &format!("{coeff}{{@}}{}", render_position(&tm.exponent, c));
        first = false;
    }
    if !tail.is_zero() {
        if !first {
            result += ",";
        }
        result += &format!("{}{{@}}0", render_veblen_coeff(&tail, c));
    }
    result += ")";
    result
}

fn psi0_to_veblen(alpha: &Term, c: &mut Caches) -> String {
    if alpha.is_zero() {
        return "1".to_string();
    }
    let t_alpha = compute_t(alpha, c);
    render_array(&t_alpha, false, c)
}

fn render_veblen_rec(q: &Term, c: &mut Caches) -> String {
    if q.is_zero() {
        return "0".to_string();
    }
    if is_finite(q) {
        let len = length1(q);
        return if len <= 1 {
            "1".to_string()
        } else {
            len.to_string()
        };
    }
    if !lt(q, &bho()) {
        return String::new();
    }

    let (head, tail) = split(q, &q.head());
    let (h0, h1, h2) = {
        let (x, y, z) = head.parts();
        (x.clone(), y.clone(), z.clone())
    };
    if !h0.is_zero() {
        return String::new(); // not ψ₀
    }
    let mut result = psi0_to_veblen(&h1, c);
    if result.is_empty() {
        return String::new();
    }

    let mut cur = h2;
    while !cur.is_zero() {
        let (c0, c1, c2) = {
            let (x, y, z) = cur.parts();
            (x.clone(), y.clone(), z.clone())
        };
        if !c0.is_zero() || !eq(&c1, &h1) {
            break;
        }
        let dup = psi0_to_veblen(&c1, c);
        if dup.is_empty() {
            return String::new();
        }
        result += &format!("+{dup}");
        cur = c2;
    }

    if !cur.is_zero() {
        let rest = render_veblen_rec(&cur, c);
        if rest.is_empty() {
            return String::new();
        }
        result += &format!("+{rest}");
    }
    if !tail.is_zero() {
        let rest = render_veblen_rec(&tail, c);
        if rest.is_empty() {
            return String::new();
        }
        result += &format!("+{rest}");
    }
    result
}

/// The ordinal as an extended Veblen expression, in LaTeX.
///
/// `None` at or above the Bachmann–Howard ordinal, where the Veblen normal form
/// stops existing; the caller falls back to the BOCF form.
pub fn veblen_latex(n: &Btbm, expr: &Expr) -> Option<String> {
    let term = term_of(n, expr)?;
    let s = render_veblen_rec(&term, &mut Caches::default());
    (!s.is_empty()).then_some(s)
}

/// The reference implementation's own ASCII rendering, for diffing.
#[cfg(test)]
fn bocf_ascii(n: &Btbm, expr: &Expr) -> Option<String> {
    convert(n, expr, Dialect::Ascii)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(n: &Btbm, s: &str) -> Expr {
        n.from_display(s).expect("test input")
    }

    /// Values checked against the reference implementation's own output; the
    /// ASCII column is exactly what it printed.
    #[test]
    fn matches_the_reference() {
        let n = Btbm::new();
        let cases = [
            ("(0)", "1", "1"),
            ("(0)(1)", "w", "\\omega"),
            ("(0)(1)(0)(1)", "w2", "\\omega\\cdot 2"),
            ("(0)(1)(1)(1)", "w^3", "\\omega^{3}"),
            ("(0)(1)(2)", "w^w", "\\omega^{\\omega}"),
            ("(0)(1)(2)(1)", "w^w+1", "\\omega^{\\omega+1}"),
            ("(0)(1,1)", "psi(Omega)", "\\psi(\\Omega)"),
            ("(0)(1,1,1)", "psi(Omega_w)", "\\psi(\\Omega_{\\omega})"),
            ("(0)(1,1)(1,1)", "psi(Omega2)", "\\psi(\\Omega\\cdot 2)"),
            (
                "(0,0,0)(1,1,1)(2,1,1)(3,1,0)",
                "psi(Omega_Omega)",
                "\\psi(\\Omega_{\\Omega})",
            ),
            (
                "(0,0,0)(1,1,1)(2,1,1)(3,1,0)(1,1,0)",
                "psi(Omega_Omega+Omega)",
                "\\psi(\\Omega_{\\Omega}+\\Omega)",
            ),
        ];
        for (src, ascii, latex) in cases {
            let e = term(&n, src);
            assert_eq!(
                bocf_ascii(&n, &e).as_deref(),
                Some(ascii),
                "ascii mismatch for {src}"
            );
            assert_eq!(
                bocf_latex(&n, &e).as_deref(),
                Some(latex),
                "latex mismatch for {src}"
            );
        }
    }

    /// Past the OCF's reach the caller must fall back, never guess.
    #[test]
    fn falls_back_past_the_limit() {
        let n = Btbm::new();
        for src in [
            "(0,0,0)(1,1,1)(2,1,1)(3,1,0)(2,0,0)",
            "(0,0,0)(1,1,1)(2,1,1)(3,1,0)(2,1,0)",
            "(0)(1^(2))",
            "(0)(1)(1^(2))",
        ] {
            assert!(
                bocf_latex(&n, &term(&n, src)).is_none(),
                "{src} should be out of range"
            );
        }
    }

    /// Width cap: a wider matrix is cut to a prefix and marked, not refused.
    #[test]
    fn truncates_when_too_wide() {
        let n = Btbm::new();
        let fits = "(0)".repeat(MAX_COLUMNS);
        assert_eq!(bocf_latex(&n, &term(&n, &fits)).as_deref(), Some("60"));

        let wide = "(0)".repeat(MAX_COLUMNS + 1);
        assert_eq!(bocf_latex(&n, &term(&n, &wide)).as_deref(), Some("60"));
    }

    /// Every convertible ordinal of the LNGI walk must convert quickly, and the
    /// ones past the cap must fall back.
    #[test]
    fn walk_stays_cheap() {
        use crate::get_time_inv;
        use crate::Lngi;
        use std::time::Instant;

        let mut l = Lngi::new();
        let n = Btbm::new();
        let mut worst = 0.0f64;
        for i in 0..200 {
            let v = 2.0 + 2.0 * (i as f64) / 200.0;
            let e = l.ordinal_for_value(v).expr;
            let start = Instant::now();
            let out = bocf_latex(&n, &e);
            worst = worst.max(start.elapsed().as_secs_f64() * 1e3);
            if let Some(s) = out {
                assert!(s.len() < 2000, "output too long at v={v}: {} chars", s.len());
            }
            let _ = get_time_inv(v);
        }
        // Debug builds are an order of magnitude slower; the shipped wasm is
        // a release build, and that is the bound that matters.
        let budget = if cfg!(debug_assertions) { 250.0 } else { 20.0 };
        assert!(worst < budget, "worst conversion was {worst:.1} ms");
    }

    /// Extended Veblen, checked against the values the notation must produce:
    /// ω², ω^ω, ε₀, ζ₀, Γ₀, the first Γ fixed point, LVO.
    #[test]
    fn veblen_matches_the_reference() {
        let n = Btbm::new();
        let cases = [
            ("(0)", "1"),
            ("(0)(1)", "\\omega"),
            ("(0)(1)(1)", "\\omega^{2}"),
            ("(0)(1)(2)", "\\omega^{\\omega}"),
            ("(0)(1)(2)(3)", "\\omega^{\\omega^{\\omega}}"),
            ("(0)(1,1)", "\\varepsilon_{0}"),
            ("(0)(1,1)(2)", "\\varepsilon_{\\omega}"),
            ("(0)(1,1)(2)(1,1)", "\\varepsilon_{\\omega+1}"),
            ("(0)(1,1)(2,1)", "\\zeta_{0}"),
            ("(0)(1,1)(2,1)(3)", "\\varphi(\\omega,0)"),
            ("(0)(1,1)(2,1)(3,1)", "\\Gamma_{0}"),
            ("(0)(1,1)(2,1)(3,1)(4)", "\\varphi(1{@}\\omega)"),
            ("(0)(1,1)(2,1)(3)(4,4)", "\\varphi(\\varepsilon_{0},0)"),
            ("(0)(1,1)(2,1)(3)(4,4)(5,4)", "\\varphi(\\zeta_{0},0)"),
            ("(0)(1,1)(2,1)(3,1)(4,1)", "\\varphi(1{@}(1,0))"),
        ];
        for (src, want) in cases {
            assert_eq!(
                veblen_latex(&n, &term(&n, src)).as_deref(),
                Some(want),
                "veblen mismatch for {src}"
            );
        }
    }

    /// The Veblen normal form stops at the Bachmann–Howard ordinal; from there
    /// the caller must show BOCF instead, never a wrong term.
    #[test]
    fn veblen_declines_from_bho_up() {
        let n = Btbm::new();
        for src in ["(0)(1,1)(2,2)", "(0)(1,1,1)", "(0)(1^(2))"] {
            assert_eq!(veblen_latex(&n, &term(&n, src)), None, "{src}");
        }
    }

    #[test]
    fn veblen_stays_cheap() {
        use crate::Lngi;
        use std::time::Instant;

        let mut l = Lngi::new();
        let n = Btbm::new();
        let mut worst = 0.0f64;
        for i in 0..200 {
            // 3.75 is BHO, the last value with a Veblen form.
            let v = 2.0 + 1.75 * (i as f64) / 200.0;
            let e = l.ordinal_for_value(v).expr;
            let start = Instant::now();
            let out = veblen_latex(&n, &e);
            worst = worst.max(start.elapsed().as_secs_f64() * 1e3);
            if let Some(s) = out {
                assert!(s.len() < 2000, "output too long at v={v}: {} chars", s.len());
            }
        }
        let budget = if cfg!(debug_assertions) { 250.0 } else { 20.0 };
        assert!(worst < budget, "worst conversion was {worst:.1} ms");
    }
}


