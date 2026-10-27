//! Native translation of the locked 0.5.1 parser, including its output quirks.
//! UTF-16 intermediates retain JavaScript indexing semantics; no JavaScript runs here.
use serde::Deserialize;
use std::sync::OnceLock;

pub const MAX_INPUT_BYTES: usize = 8192;
pub const MAX_OUTPUT_UNITS: usize = 131072;
const MAX_DEPTH: usize = 96;
const MAX_WORK: usize = 250000;
pub const PARSE_ERROR: &str = "Error parsing expression";

#[derive(Clone, Default, Debug, PartialEq)]
struct Js(Vec<u16>);
impl From<&str> for Js {
    fn from(s: &str) -> Self {
        Self(s.encode_utf16().collect())
    }
}
impl From<String> for Js {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}
impl From<&Js> for Js {
    fn from(s: &Js) -> Self {
        s.clone()
    }
}
macro_rules! j { ($($x:expr),* $(,)?) => {{ let mut s=Js::default(); $(s.0.extend(Js::from($x).0);)* s }}; }
impl Js {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn at(&self, i: usize) -> u16 {
        self.0.get(i).copied().unwrap_or(0xffff)
    }
    fn is(&self, s: &str) -> bool {
        self.0.iter().copied().eq(s.encode_utf16())
    }
    fn starts(&self, s: &str) -> bool {
        self.0.starts_with(&Self::from(s).0)
    }
    fn range(&self, a: usize, b: usize) -> Self {
        let (a, b) = (a.min(self.len()), b.min(self.len()));
        Self(self.0[a.min(b)..a.max(b)].to_vec())
    }
    fn sub(&self, start: isize, len: usize) -> Self {
        let a = if start < 0 {
            self.len().saturating_sub(start.unsigned_abs())
        } else {
            (start as usize).min(self.len())
        };
        Self(self.0[a..a.saturating_add(len).min(self.len())].to_vec())
    }
    fn tail(&self, start: usize) -> Self {
        self.sub(start as isize, usize::MAX)
    }
}
const CONST: i32 = 0;
const UNARY: i32 = 1;
const BINARY: i32 = 2;
const INFIX: i32 = 3;
const LEFT: i32 = 4;
const RIGHT: i32 = 5;
const SPACE: i32 = 6;
const UNDER: i32 = 7;
const DEF: i32 = 8;
const BOTH: i32 = 9;
const TEXT: i32 = 10;
#[derive(Clone, Deserialize)]
struct RawSymbol {
    input: String,
    tag: String,
    output: Option<String>,
    tex: Option<String>,
    ttype: Option<i32>,
    #[serde(default)]
    val: bool,
    #[serde(default)]
    invisible: bool,
    #[serde(default, rename = "func")]
    function: bool,
    #[serde(default)]
    acc: bool,
    rewriteleftright: Option<[String; 2]>,
}
#[derive(Clone)]
struct Symbol {
    input: Js,
    tag: String,
    output: Option<Js>,
    tex: Js,
    kind: i32,
    invisible: bool,
    function: bool,
    acc: bool,
    rewrite: Option<[Js; 2]>,
}
fn symbols() -> &'static [Symbol] {
    static SYMBOLS: OnceLock<Vec<Symbol>> = OnceLock::new();
    SYMBOLS.get_or_init(|| {
        let raw: Vec<RawSymbol> =
            serde_json::from_str(include_str!("symbols.json")).expect("immutable symbol table");
        raw.into_iter()
            .map(|r| {
                let tex = j!(
                    if r.val { "" } else { "\\" },
                    r.tex.as_deref().unwrap_or(&r.input)
                );
                Symbol {
                    input: Js::from(r.input),
                    tag: r.tag,
                    output: r.output.map(Js::from),
                    tex,
                    kind: r.ttype.unwrap_or(-1),
                    invisible: r.invisible,
                    function: r.function,
                    acc: r.acc,
                    rewrite: r.rewriteleftright.map(|r| r.map(Js::from)),
                }
            })
            .collect()
    })
}
fn remove(s: &Js, n: usize) -> Js {
    let skip = usize::from(s.at(n) == 92 && s.at(n + 1) != 92 && s.at(n + 1) != 32);
    let mut a = (n + skip).min(s.len());
    while a < s.len() && s.at(a) <= 32 {
        a += 1;
    }
    s.tail(a)
}
fn unbracket(mut s: Js) -> Js {
    if s.at(0) == 123 && s.at(s.len().saturating_sub(1)) == 125 {
        let chop = if s.sub(1, 5).is("\\left") {
            if [40, 91, 123].contains(&s.at(6)) {
                7
            } else if s.sub(6, 7).is("\\lbrace") {
                13
            } else {
                0
            }
        } else if [40, 91].contains(&s.at(1)) {
            2
        } else {
            0
        };
        if chop > 0 {
            let end = s.sub(s.len() as isize - 8, usize::MAX);
            if ["\\right)}", "\\right]}", "\\right.}"]
                .iter()
                .any(|x| end.is(x))
            {
                s = j!("{", s.tail(chop));
                s = j!(s.sub(0, s.len().saturating_sub(8)), "}");
            } else if end.is("\\rbrace}") {
                s = j!("{", s.tail(chop));
                s = j!(s.sub(0, s.len().saturating_sub(14)), "}");
            }
        }
    }
    s
}
#[derive(Default)]
struct Parser {
    nesting: usize,
    depth: usize,
    work: usize,
    current: i32,
    limited: bool,
}
type Parsed = Result<(Option<Js>, Js), ()>;
fn value(v: Option<Js>) -> Js {
    v.unwrap_or_else(|| Js::from("null"))
}
impl Parser {
    fn tick(&mut self, n: usize) -> Result<(), ()> {
        self.work = self.work.saturating_add(n);
        if self.work > MAX_WORK || self.depth > MAX_DEPTH {
            self.limited = true;
            Err(())
        } else {
            Ok(())
        }
    }
    fn symbol(&mut self, s: &Js) -> Result<Symbol, ()> {
        self.tick(1)?;
        let previous = self.current;
        if let Some(sym) = symbols()
            .iter()
            .rev()
            .filter(|a| s.0.starts_with(&a.input.0))
            .max_by_key(|a| a.input.len())
        {
            self.current = sym.kind;
            return Ok(sym.clone());
        }
        self.current = CONST;
        let mut k = 0;
        while (48..=57).contains(&s.at(k)) {
            k += 1;
        }
        let mut end = k;
        if s.at(k) == 46 && (48..=57).contains(&s.at(k + 1)) {
            k += 1;
            while (48..=57).contains(&s.at(k)) {
                k += 1;
            }
            end = k;
        }
        if end == 0 {
            end = usize::from(s.len() > 0);
        }
        let input = s.sub(0, end);
        let letter = (65..=90).contains(&input.at(0)) || (97..=122).contains(&input.at(0));
        let numeric = end > 1 || (48..=57).contains(&input.at(0));
        let minus = input.is("-") && previous == INFIX;
        if minus {
            self.current = INFIX;
        }
        Ok(Symbol {
            tex: input.clone(),
            output: Some(input.clone()),
            input,
            tag: if numeric {
                "mn"
            } else if letter {
                "mi"
            } else {
                "mo"
            }
            .into(),
            kind: if minus { UNARY } else { CONST },
            function: minus,
            invisible: false,
            acc: false,
            rewrite: None,
        })
    }
    fn simple(&mut self, s: Js) -> Parsed {
        self.depth += 1;
        self.tick(1)?;
        let r = self.simple_inner(s);
        self.depth -= 1;
        r
    }
    fn simple_inner(&mut self, mut s: Js) -> Parsed {
        s = remove(&s, 0);
        let mut sym = self.symbol(&s)?;
        if sym.kind == RIGHT && self.nesting > 0 {
            return Ok((None, s));
        }
        if sym.kind == DEF {
            s = j!(sym.output.as_ref().ok_or(())?, remove(&s, sym.input.len()));
            sym = self.symbol(&s)?;
        }
        match sym.kind {
            CONST | UNDER => {
                s = remove(&s, sym.input.len());
                let node = if sym.tex.starts("\\") || sym.tag == "mo" {
                    sym.tex
                } else {
                    j!("{", sym.tex, "}")
                };
                Ok((Some(node), s))
            }
            LEFT => {
                self.nesting += 1;
                s = remove(&s, sym.input.len());
                let (mut node, tail) = self.expr(s, true)?;
                self.nesting -= 1;
                let chop = if node.starts("\\right") {
                    if [40, 91, 125].contains(&node.at(6)) {
                        6
                    } else if node.at(6) == 46 {
                        7
                    } else if node.sub(6, 7).is("\\rbrace") {
                        13
                    } else {
                        0
                    }
                } else {
                    0
                };
                node = if chop > 0 {
                    j!(
                        "{",
                        if sym.invisible {
                            Js::default()
                        } else {
                            sym.tex
                        },
                        node.tail(chop),
                        "}"
                    )
                } else {
                    j!(
                        "{\\left",
                        if sym.invisible {
                            Js::from(".")
                        } else {
                            sym.tex
                        },
                        node,
                        "}"
                    )
                };
                Ok((Some(node), tail))
            }
            TEXT => {
                let quote = sym.input.is("\"");
                if !quote {
                    s = remove(&s, sym.input.len());
                }
                let close = match s.at(0) {
                    123 => Some(125),
                    40 => Some(41),
                    91 => Some(93),
                    _ => None,
                };
                let i = if let Some(c) = close {
                    s.0.iter().position(|x| *x == c).unwrap_or(s.len())
                } else if quote {
                    s.0.iter()
                        .skip(1)
                        .position(|x| *x == 34)
                        .map(|x| x + 1)
                        .unwrap_or(0)
                } else {
                    0
                };
                let st = if i < 1 { Js::default() } else { s.range(1, i) };
                let node = j!(
                    if st.at(0) == 32 { "\\ " } else { "" },
                    "\\text{",
                    &st,
                    "}",
                    if st.at(st.len().saturating_sub(1)) == 32 {
                        "\\ "
                    } else {
                        ""
                    }
                );
                Ok((Some(node), remove(&s, i + 1)))
            }
            UNARY => {
                s = remove(&s, sym.input.len());
                let (node, tail) = self.simple(s.clone())?;
                let Some(mut node) = node else {
                    return Ok((Some(j!("{", sym.tex, "}")), s));
                };
                if sym.function {
                    let st = s.at(0);
                    let word = sym.input.len() == 1
                        && (sym.input.at(0) == 95
                            || (48..=57).contains(&sym.input.at(0))
                            || (65..=90).contains(&sym.input.at(0))
                            || (97..=122).contains(&sym.input.at(0)));
                    if [94, 95, 47, 124, 44].contains(&st) || (word && st != 40) {
                        return Ok((Some(j!("{", sym.tex, "}")), s));
                    }
                    return Ok((Some(j!("{", sym.tex, "{", node, "}}")), tail));
                }
                node = unbracket(node);
                let node = if sym.input.is("sqrt") {
                    j!("\\sqrt{", node, "}")
                } else if sym.input.is("cancel") {
                    j!("\\cancel{", node, "}")
                } else if let Some([l, r]) = sym.rewrite {
                    j!("{\\left", l, node, "\\right", r, "}")
                } else if sym.acc {
                    j!(sym.tex, "{", node, "}")
                } else {
                    j!("{", sym.tex, "{", node, "}}")
                };
                Ok((Some(node), tail))
            }
            BINARY => {
                s = remove(&s, sym.input.len());
                let (a, tail) = self.simple(s.clone())?;
                let Some(a) = a else {
                    return Ok((Some(j!("{", sym.tex, "}")), s));
                };
                let a = unbracket(a);
                let (b, tail) = self.simple(tail)?;
                let Some(b) = b else {
                    return Ok((Some(j!("{", sym.tex, "}")), s));
                };
                let b = unbracket(b);
                let node = if sym.input.is("color") {
                    let a = Js(a.0.into_iter().filter(|x| *x != 123 && *x != 125).collect());
                    j!("{\\color{", a, "}", b, "}")
                } else if sym.input.is("root") {
                    j!("{\\sqrt[", a, "]{", b, "}}")
                } else {
                    j!("{", sym.tex, "{", a, "}{", b, "}}")
                };
                Ok((Some(node), tail))
            }
            INFIX => Ok((sym.output, remove(&s, sym.input.len()))),
            SPACE => Ok((
                Some(j!("{\\quad\\text{", sym.input.clone(), "}\\quad}")),
                remove(&s, sym.input.len()),
            )),
            BOTH => {
                self.nesting += 1;
                s = remove(&s, sym.input.len());
                let (node, tail) = self.expr(s.clone(), false)?;
                self.nesting -= 1;
                if node.at(node.len().saturating_sub(1)) == 124 {
                    Ok((Some(j!("{\\left|", node, "}")), tail))
                } else {
                    Ok((Some(Js::from("{\\mid}")), s))
                }
            }
            _ => Ok((Some(j!("{", sym.tex, "}")), remove(&s, sym.input.len()))),
        }
    }
    fn intermediate(&mut self, s: Js) -> Parsed {
        self.depth += 1;
        self.tick(1)?;
        let r = self.intermediate_inner(s);
        self.depth -= 1;
        r
    }
    fn intermediate_inner(&mut self, mut s: Js) -> Parsed {
        s = remove(&s, 0);
        let first = self.symbol(&s)?;
        let (mut node, tail) = self.simple(s)?;
        s = tail;
        let sym = self.symbol(&s)?;
        if sym.kind == INFIX && !sym.input.is("/") {
            s = remove(&s, sym.input.len());
            let (a, tail) = self.simple(s)?;
            let a = a.map(unbracket).unwrap_or_else(|| Js::from("{}"));
            s = tail;
            if sym.input.is("_") {
                let second = self.symbol(&s)?;
                if second.input.is("^") {
                    s = remove(&s, second.input.len());
                    let (b, tail) = self.simple(s)?;
                    // Original removeBrackets(null) throws here.
                    let b = unbracket(b.ok_or(())?);
                    s = tail;
                    node = Some(j!("{", value(node), "_{", a, "}^{", b, "}", "}"));
                } else {
                    node = Some(j!(value(node), "_{", a, "}"));
                }
            } else {
                node = Some(j!(value(node), "^{", a, "}"));
            }
            if first.function {
                let second = self.symbol(&s)?;
                if second.kind != INFIX && second.kind != RIGHT {
                    let (b, tail) = self.intermediate(s)?;
                    node = Some(j!("{", value(node), value(b), "}"));
                    s = tail;
                }
            }
        }
        Ok((node, s))
    }
    fn expr(&mut self, mut s: Js, right: bool) -> Result<(Js, Js), ()> {
        self.depth += 1;
        self.tick(1)?;
        let mut frag = Js::default();
        let sym = loop {
            s = remove(&s, 0);
            let (node, tail) = self.intermediate(s)?;
            s = tail;
            let mut sym = self.symbol(&s)?;
            if sym.kind == INFIX && sym.input.is("/") {
                s = remove(&s, sym.input.len());
                let (den, tail) = self.intermediate(s)?;
                s = tail;
                let den = den.map(unbracket).unwrap_or_else(|| Js::from("{}"));
                frag = j!(frag, "\\frac{", unbracket(node.ok_or(())?), "}{", den, "}");
                sym = self.symbol(&s)?;
            } else {
                frag = j!(frag, value(node));
            }
            self.tick(frag.len() + 1)?;
            if frag.len() > MAX_OUTPUT_UNITS {
                self.limited = true;
                return Err(());
            }
            if !(((sym.kind != RIGHT && (sym.kind != BOTH || right)) || self.nesting == 0)
                && sym.output.as_ref().is_none_or(|x| x.len() != 0))
            {
                break sym;
            }
        };
        let added = sym.kind == RIGHT || sym.kind == BOTH;
        if added {
            frag = self.matrix(frag, &sym)?;
            s = remove(&s, sym.input.len());
            frag = j!(
                frag,
                "\\right",
                if sym.invisible {
                    Js::from(".")
                } else {
                    sym.tex
                }
            );
        }
        if self.nesting > 0 && !added {
            frag = j!(frag, "\\right.");
        }
        self.depth -= 1;
        Ok((frag, s))
    }
    fn matrix(&mut self, frag: Js, sym: &Symbol) -> Result<Js, ()> {
        let len = frag.len();
        if len <= 2 || frag.at(0) != 123 || !frag.0.contains(&44) {
            return Ok(frag);
        }
        let r = frag.at(len - 2);
        let l = frag.at(6);
        if !((l == 40 && r == 41 && !sym.output.as_ref().is_some_and(|x| x.is("}")))
            || (l == 91 && r == 93))
        {
            return Ok(frag);
        }
        let mut out = Js::from("\\begin{matrix}");
        let mut pos = vec![0];
        let mut sub = vec![vec![0]; len + 1];
        let (mut nesting, mut any, mut last, mut valid) = (0i32, 0i32, 0usize, true);
        let mut i = 1;
        while i < len - 1 {
            self.tick(1)?;
            let c = frag.at(i);
            if c == l {
                nesting += 1;
            }
            if c == r {
                nesting -= 1;
                if nesting == 0 && frag.at(i + 2) == 44 && frag.at(i + 3) == 123 {
                    pos.push(i + 2);
                    last = i + 2;
                    sub[last] = vec![last];
                }
            }
            if [91, 40, 123].contains(&c) {
                any += 1;
            }
            if [93, 41, 125].contains(&c) {
                any -= 1;
            }
            if c == 44 && any == 1 {
                sub[last].push(i);
            }
            if any < 0 {
                if last == i + 1 {
                    i += 1;
                } else {
                    valid = false;
                }
            }
            i += 1;
        }
        pos.push(len);
        let mut last_count = None;
        if nesting == 0 && valid {
            for i in 0..pos.len() - 1 {
                if i > 0 {
                    out = j!(out, "\\\\");
                }
                let start = pos[i];
                let end = pos[i + 1];
                let offset = if i == 0 { 7 } else { 8 };
                let mut row = vec![];
                if sub[start].len() == 1 {
                    row.push(frag.sub(
                        (start + offset) as isize,
                        end.saturating_sub(start + offset + 8),
                    ));
                } else {
                    row.push(frag.range(start + offset, sub[start][1]));
                    for j in 2..sub[start].len() {
                        row.push(frag.range(sub[start][j - 1] + 1, sub[start][j]));
                    }
                    row.push(
                        frag.range(sub[start][sub[start].len() - 1] + 1, end.saturating_sub(8)),
                    );
                }
                if last_count.is_some_and(|n| n > 0 && n != row.len()) {
                    valid = false;
                } else if last_count.is_none() {
                    last_count = Some(row.len());
                }
                for (j, cell) in row.into_iter().enumerate() {
                    if j > 0 {
                        out = j!(out, "&");
                    }
                    out = j!(out, cell);
                }
            }
        }
        out = j!(out, "\\end{matrix}");
        Ok(if valid { out } else { frag })
    }
}
fn js_space(c: char) -> bool {
    matches!(c,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}
/// Bounds reject the whole expression; never silently truncate. Library parse errors
/// become the exact source helper literal. Resource-limit failures are distinct.
pub fn convert(input: &str) -> Result<String, &'static str> {
    if input.len() > MAX_INPUT_BYTES {
        return Err("Expression exceeds 8192 UTF-8 bytes");
    }
    let s = input
        .replace("&nbsp;", "")
        .replace('\u{00a0}', "")
        .replace("&#160;", "")
        .replace("&gt;", ">")
        .replace("&lt;", "<");
    if s.chars().all(js_space) {
        return Ok(String::new());
    }
    let mut parser = Parser::default();
    let result = parser.expr(Js::from(s.trim_start_matches(js_space)), false);
    if parser.limited {
        return Err("Expression exceeds parser depth/work limit");
    }
    match result {
        Ok((out, _)) if out.len() <= MAX_OUTPUT_UNITS => String::from_utf16(&out.0)
            .map_err(|_| "Library output contains an unpaired UTF-16 surrogate"),
        Ok(_) => Err("Expression exceeds output limit"),
        Err(()) => Ok(PARSE_ERROR.into()),
    }
}
pub fn wrapped(latex: &str, style: &str) -> String {
    match style {
        "double" => format!("$${latex}$$"),
        "single" => format!("${latex}$"),
        _ => latex.into(),
    }
}
