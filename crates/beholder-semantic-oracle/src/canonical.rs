// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-signatures
// ---
//! Canonical spelling of a signature's type text, per language.
//!
//! A type text is the language's own type syntax with `@{id}` for each name
//! that denotes an entity. This module tokenizes a text, applies the sort and
//! spelling rules the language's profile states, and renders the one spelling.
//! It needs no type grammar: the rules read tokens and bracket nesting. The
//! validator calls [`canonicalize`] now, and an adapter can call it on the type
//! text it reports.

use logos::Logos;

/// What a text's `@{id}` names, for the rules that rewrite a spelling by its
/// target, such as TypeScript's `Array<string>`.
pub trait NameResolver {
    /// The canonical path of an external record `id` names, for the rules that
    /// rewrite a standard declaration. A project entity of the same name is
    /// another type, so it returns `None` here.
    fn standard_path<'r>(&'r self, id: &'r str) -> Option<&'r str>;

    /// The `language-kind` of the record `id` names, where it is known.
    fn language_kind(&self, _id: &str) -> Option<&str> {
        None
    }
}

/// A resolver for a text whose `@{...}` already holds a path.
pub struct PathsAreIds;

impl NameResolver for PathsAreIds {
    fn standard_path<'r>(&'r self, id: &'r str) -> Option<&'r str> {
        Some(id)
    }
}

/// The result of canonicalizing one type text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Canonical {
    /// The one spelling.
    pub text: String,
    /// The ids its `@{id}` names, outside string literals.
    pub ids: Vec<String>,
    /// Words that are neither predeclared nor syntax nor a label. Each is a
    /// type parameter, or a name that the author did not write as `@{id}`.
    pub bare: Vec<String>,
    /// Reasons the text is not one spelling that sorting and spacing can fix.
    pub issues: Vec<String>,
}

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n]+")]
enum Lexeme {
    #[regex(r"@\{[^}]*\}")]
    Name,
    #[regex(r"@\?\{[^}]*\}")]
    Unresolved,
    #[regex(r#""([^"\\]|\\.)*""#)]
    #[regex(r"`[^`]*`")]
    Text,
    #[regex(r"'([^'\\]|\\[^']+)'")]
    Char,
    #[regex(r"'[A-Za-z_][A-Za-z0-9_]*")]
    Lifetime,
    #[regex(r"[A-Za-z_$][A-Za-z0-9_$]*")]
    Word,
    #[regex(r"[0-9][0-9A-Za-z_.]*")]
    Number,
    #[token("->")]
    #[token("=>")]
    #[token("::")]
    #[token("...")]
    Operator,
    #[regex(r".", priority = 0)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Token(Lexeme, String),
    Group(char, Vec<Node>),
}

/// The closing bracket of an opening bracket.
fn closing(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => '>',
    }
}

/// The per-language tables the rules read.
struct Language {
    name: &'static str,
    /// Whether `<` and `>` are brackets.
    angle: bool,
    /// Names written bare.
    predeclared: &'static [&'static str],
    /// Words of the type syntax.
    keywords: &'static [&'static str],
    /// Set operators, sorted inside one operand run.
    set_operators: &'static [&'static str],
    /// Words after which `(`, `[` or `<` takes a space.
    spaced_keywords: &'static [&'static str],
    /// Whether `&` and `*` are prefixes, with no space after.
    prefix_references: bool,
    /// Whether `{` holds its content with no inner space.
    tight_braces: bool,
}

const RUST: Language = Language {
    name: "rust",
    angle: true,
    predeclared: &[
        "bool", "char", "str", "f16", "f32", "f64", "f128", "i8", "i16", "i32", "i64", "i128",
        "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    ],
    keywords: &[
        "as", "async", "const", "dyn", "extern", "fn", "for", "impl", "mut", "Self", "unsafe",
        "use", "where",
    ],
    set_operators: &["+"],
    spaced_keywords: &["as", "const", "dyn", "impl", "mut", "unsafe", "extern"],
    prefix_references: true,
    tight_braces: false,
};

const TYPESCRIPT: Language = Language {
    name: "typescript",
    angle: true,
    predeclared: &[
        "any",
        "bigint",
        "boolean",
        "false",
        "never",
        "null",
        "number",
        "object",
        "string",
        "symbol",
        "this",
        "true",
        "undefined",
        "unknown",
        "void",
    ],
    keywords: &[
        "abstract", "asserts", "extends", "in", "infer", "is", "keyof", "new", "out", "readonly",
        "typeof", "unique",
    ],
    set_operators: &["&", "|"],
    spaced_keywords: &[
        "abstract", "asserts", "extends", "in", "infer", "is", "keyof", "new", "out", "readonly",
        "typeof", "unique",
    ],
    prefix_references: false,
    tight_braces: false,
};

const GO: Language = Language {
    name: "go",
    angle: false,
    predeclared: &[
        "any",
        "bool",
        "comparable",
        "complex64",
        "complex128",
        "error",
        "float32",
        "float64",
        "int",
        "int8",
        "int16",
        "int32",
        "int64",
        "string",
        "uint",
        "uint8",
        "uint16",
        "uint32",
        "uint64",
        "uintptr",
        "untyped",
    ],
    keywords: &["chan", "func", "interface", "map", "struct"],
    set_operators: &["|"],
    spaced_keywords: &["chan", "interface", "struct"],
    prefix_references: true,
    tight_braces: false,
};

const PYTHON: Language = Language {
    name: "python",
    angle: false,
    predeclared: &[
        "bool",
        "bytearray",
        "bytes",
        "complex",
        "dict",
        "float",
        "frozenset",
        "int",
        "list",
        "None",
        "object",
        "set",
        "str",
        "tuple",
        "type",
    ],
    keywords: &[],
    set_operators: &["|"],
    spaced_keywords: &[],
    prefix_references: false,
    tight_braces: false,
};

const CSHARP: Language = Language {
    name: "csharp",
    angle: true,
    predeclared: &[
        "bool", "byte", "char", "decimal", "double", "dynamic", "float", "int", "long", "nint",
        "nuint", "object", "sbyte", "short", "string", "uint", "ulong", "ushort", "void",
    ],
    keywords: &[
        "allows",
        "class",
        "default",
        "delegate",
        "managed",
        "new",
        "notnull",
        "ref",
        "readonly",
        "struct",
        "unmanaged",
    ],
    set_operators: &[],
    spaced_keywords: &["allows", "ref", "readonly"],
    prefix_references: true,
    tight_braces: false,
};

const DART: Language = Language {
    name: "dart",
    angle: true,
    predeclared: &[
        "bool", "double", "dynamic", "int", "Never", "Null", "num", "Object", "String", "void",
    ],
    keywords: &["Function", "required", "covariant"],
    set_operators: &[],
    spaced_keywords: &["required", "covariant"],
    prefix_references: false,
    tight_braces: true,
};

fn language(name: &str) -> Option<&'static Language> {
    [&RUST, &TYPESCRIPT, &GO, &PYTHON, &CSHARP, &DART]
        .into_iter()
        .find(|language| language.name == name)
}

/// The names a language writes bare in a type text.
pub fn predeclared(name: &str) -> &'static [&'static str] {
    language(name).map_or(&[], |language| language.predeclared)
}

/// The ids that `@{id}` names in a type or value text. A string literal names
/// nothing, and a char literal's quote opens no string.
pub fn named_ids(language: &str, text: &str) -> Vec<String> {
    let source = quoted_for(language, text);
    let mut lexer = Lexeme::lexer(&source);
    let mut ids = Vec::new();
    while let Some(lexeme) = lexer.next() {
        if lexeme == Ok(Lexeme::Name) {
            let span = lexer.slice();
            ids.push(span[2..span.len() - 1].to_owned());
        }
    }
    ids
}

/// A text with its string literals double-quoted, in the languages that also
/// write them with single quotes. Rust's and C#'s `'` is a char or a lifetime.
fn quoted_for(language: &str, text: &str) -> String {
    if matches!(language, "typescript" | "python") {
        normalize_quotes(text)
    } else {
        text.to_owned()
    }
}

fn tokens(text: &str) -> Vec<(Lexeme, String)> {
    let mut lexer = Lexeme::lexer(text);
    let mut out = Vec::new();
    while let Some(lexeme) = lexer.next() {
        out.push((lexeme.unwrap_or(Lexeme::Other), lexer.slice().to_owned()));
    }
    out
}

/// Nests tokens by bracket. An unbalanced text stays flat and reports an issue.
fn nest(tokens: Vec<(Lexeme, String)>, language: &Language, issues: &mut Vec<String>) -> Vec<Node> {
    let mut stack: Vec<(char, Vec<Node>)> = vec![('\0', Vec::new())];
    for (lexeme, text) in tokens {
        let bracket = (lexeme == Lexeme::Other)
            .then(|| text.chars().next())
            .flatten();
        match bracket {
            Some(open @ ('(' | '[' | '{')) => stack.push((open, Vec::new())),
            Some('<') if language.angle => stack.push(('<', Vec::new())),
            Some(close @ (')' | ']' | '}')) => close_group(&mut stack, close, issues),
            Some('>') if language.angle => close_group(&mut stack, '>', issues),
            _ => stack
                .last_mut()
                .expect("the root frame stays")
                .1
                .push(Node::Token(lexeme, text)),
        }
    }
    while stack.len() > 1 {
        let (open, children) = stack.pop().expect("checked");
        issues.push(format!("`{open}` has no closing bracket"));
        stack
            .last_mut()
            .expect("the root frame stays")
            .1
            .push(Node::Group(open, children));
    }
    stack.pop().expect("the root frame stays").1
}

fn close_group(stack: &mut Vec<(char, Vec<Node>)>, close: char, issues: &mut Vec<String>) {
    if stack.len() > 1 && closing(stack.last().expect("checked").0) == close {
        let (open, children) = stack.pop().expect("checked");
        stack
            .last_mut()
            .expect("the root frame stays")
            .1
            .push(Node::Group(open, children));
    } else {
        issues.push(format!("`{close}` has no opening bracket"));
    }
}

fn word(node: Option<&Node>) -> Option<&str> {
    match node {
        Some(Node::Token(Lexeme::Word, text)) => Some(text),
        _ => None,
    }
}

fn is_op(node: &Node, op: &str) -> bool {
    matches!(node, Node::Token(lexeme, text)
        if text == op && matches!(lexeme, Lexeme::Other | Lexeme::Operator))
}

/// The name a standard declaration goes by, for a path the language's standard
/// library defines at that exact path.
fn standard_name<'p>(language: &str, path: &'p str) -> Option<&'p str> {
    match language {
        "typescript" => path.strip_prefix("typescript/lib/lib.es5.d.ts#"),
        "python" => path.strip_prefix("typing."),
        // A generic metadata name ends in its arity: `System.Nullable`1`.
        "csharp" => path
            .strip_prefix("System.")
            .map(|name| name.split('`').next().unwrap_or(name)),
        _ => None,
    }
}

/// A `@{id}` node's standard name. A bare word never is one, because a text
/// writes a standard declaration by its id.
fn standard_of<'n>(language: &str, node: &'n Node, names: &'n dyn NameResolver) -> Option<&'n str> {
    let Node::Token(Lexeme::Name, text) = node else {
        return None;
    };
    standard_name(language, names.standard_path(&text[2..text.len() - 1])?)
}

fn text_token(text: &str) -> Node {
    Node::Token(Lexeme::Word, text.to_owned())
}

fn op_token(text: &str) -> Node {
    Node::Token(Lexeme::Other, text.to_owned())
}

/// Splits nodes at each top-level occurrence of a separator.
fn split(nodes: &[Node], separator: &str) -> Vec<Vec<Node>> {
    nodes
        .split(|node| is_op(node, separator))
        .map(<[Node]>::to_vec)
        .collect()
}

fn has_op(nodes: &[Node], ops: &[&str]) -> bool {
    nodes
        .iter()
        .any(|node| ops.iter().any(|op| is_op(node, op)))
}

/// The bare text of nodes, in the language's spacing, for use as a sort key.
fn key(nodes: &[Node], language: &Language) -> String {
    render(nodes, language)
}

fn sorted_join(mut items: Vec<Vec<Node>>, language: &Language, op: &str) -> Vec<Node> {
    items.sort_by_key(|item| key(item, language));
    items.dedup_by_key(|item| key(item, language));
    let mut out = Vec::new();
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            out.push(op_token(op));
        }
        out.extend(item);
    }
    out
}

/// Tokens at which a run of set operands ends, per language.
fn stops(language: &Language) -> &'static [&'static str] {
    match language.name {
        "typescript" => &["=>", "?", ":", "extends", ",", ";", "="],
        "rust" => &[":", ",", ";", "=", "->"],
        _ => &[",", ";", ":", "="],
    }
}

fn is_stop(node: &Node, language: &Language) -> bool {
    stops(language).iter().any(|stop| match node {
        Node::Token(_, text) => text == stop,
        Node::Group(..) => false,
    })
}

/// Sorts the operands of each set operator inside each run between stops.
fn sort_sets(nodes: Vec<Node>, language: &Language) -> Vec<Node> {
    if language.set_operators.is_empty() {
        return nodes;
    }
    let mut out = Vec::new();
    let mut run = Vec::new();
    for node in nodes {
        if is_stop(&node, language) {
            out.extend(sort_run(std::mem::take(&mut run), language));
            out.push(node);
        } else {
            run.push(node);
        }
    }
    out.extend(sort_run(run, language));
    out
}

fn sort_run(run: Vec<Node>, language: &Language) -> Vec<Node> {
    let mut run = run;
    // A leading `|` is layout.
    if run.first().is_some_and(|node| is_op(node, "|")) {
        run.remove(0);
    }
    // The bound list of `impl` and `dyn` starts after the keyword.
    let start = run
        .iter()
        .rposition(|node| matches!(word(Some(node)), Some("impl" | "dyn")))
        .map_or(0, |at| at + 1);
    let (prefix, operands) = run.split_at(start);
    let mut out = prefix.to_vec();
    out.extend(sort_operators(operands.to_vec(), language, 0));
    out
}

fn sort_operators(nodes: Vec<Node>, language: &Language, depth: usize) -> Vec<Node> {
    // `|` binds loosest where both operators exist, so that it splits first.
    let order: &[&str] = match language.name {
        "typescript" => &["|", "&"],
        "rust" => &["+"],
        _ => &["|"],
    };
    let Some(op) = order.get(depth) else {
        return nodes;
    };
    if !has_op(&nodes, &[op]) {
        return sort_operators(nodes, language, depth + 1);
    }
    let parts = split(&nodes, op)
        .into_iter()
        .map(|part| sort_operators(part, language, depth + 1))
        .collect();
    sorted_join(parts, language, op)
}

/// Removes parentheses that only group one operand of a language that states so.
fn drop_redundant_parentheses(nodes: Vec<Node>, language: &Language) -> Vec<Node> {
    if !matches!(language.name, "rust" | "typescript" | "python") {
        return nodes;
    }
    let mut out: Vec<Node> = Vec::new();
    let mut iter = nodes.into_iter().peekable();
    while let Some(node) = iter.next() {
        let next = iter.peek();
        let redundant = match &node {
            Node::Group('(', inner) => {
                // `fn(u8)` and `Fn(u8)` call; a keyword before it does not.
                let called = match out.last() {
                    Some(Node::Token(Lexeme::Word, text)) => {
                        text == "fn" || !language.keywords.contains(&text.as_str())
                    }
                    Some(Node::Token(Lexeme::Name, _) | Node::Group(..)) => true,
                    _ => false,
                };
                let function_shape = next.is_some_and(|n| is_op(n, "=>") || is_op(n, ":"));
                let separated = inner.is_empty() || has_op(inner, &[","]);
                let operators = match language.name {
                    "rust" => has_op(inner, &["+"]),
                    "typescript" => {
                        has_op(inner, &["|", "&", "=>", "?", ":"])
                            || word(inner.first()) == Some("new")
                    }
                    // A Python group of one union only groups it: an argument
                    // separator after it is not an operator applied to it.
                    _ => false,
                };
                !called && !function_shape && !separated && !operators
            }
            _ => false,
        };
        match node {
            Node::Group('(', inner) if redundant => out.extend(inner),
            other => out.push(other),
        }
    }
    out
}

/// Language spellings a rule rewrites to one form.
fn rewrite(nodes: Vec<Node>, language: &Language, names: &dyn NameResolver) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    let mut iter = nodes.into_iter().peekable();
    while let Some(node) = iter.next() {
        let standard = standard_of(language.name, &node, names).map(str::to_owned);
        let written = word(Some(&node)).map(str::to_owned);
        let args = match iter.peek() {
            Some(Node::Group(open @ ('<' | '['), args)) if language.angle == (*open == '<') => {
                Some(args.clone())
            }
            _ => None,
        };
        let replaced = match (language.name, standard.as_deref(), args) {
            ("typescript", Some("Array"), Some(args)) => {
                iter.next();
                Some(array_of(args, None))
            }
            ("typescript", Some("ReadonlyArray"), Some(args)) => {
                iter.next();
                Some(array_of(args, Some("readonly")))
            }
            ("python", Some("Optional"), Some(args)) => {
                iter.next();
                let mut union = args;
                union.push(op_token("|"));
                union.push(text_token("None"));
                Some(union)
            }
            ("python", Some("Union"), Some(args)) => {
                iter.next();
                Some(
                    split(&args, ",")
                        .into_iter()
                        .enumerate()
                        .flat_map(|(i, part)| {
                            let separator = (i > 0).then(|| op_token("|"));
                            separator.into_iter().chain(part)
                        })
                        .collect(),
                )
            }
            ("python", Some(alias), Some(args)) if python_generic_alias(alias).is_some() => {
                iter.next();
                let lower = python_generic_alias(alias).expect("checked");
                Some(vec![text_token(lower), Node::Group('[', args)])
            }
            ("csharp", Some("Nullable"), Some(mut args)) if args.len() == 1 => {
                iter.next();
                let inner = args.pop().expect("checked");
                Some(vec![inner, op_token("?")])
            }
            ("go", _, _) => match written.as_deref() {
                Some("interface") if matches!(iter.peek(), Some(Node::Group('{', inner)) if inner.is_empty()) =>
                {
                    iter.next();
                    Some(vec![text_token("any")])
                }
                Some("byte") => Some(vec![text_token("uint8")]),
                // `untyped rune` is a constant's kind, not the alias.
                Some("rune") if word(out.last()) != Some("untyped") => {
                    Some(vec![text_token("int32")])
                }
                _ => None,
            },
            ("csharp", Some(metadata), _) if csharp_keyword(metadata).is_some() => {
                Some(vec![text_token(csharp_keyword(metadata).expect("checked"))])
            }
            _ => None,
        };
        match replaced {
            Some(nodes) => out.extend(nodes),
            None => out.push(node),
        }
    }
    out
}

fn array_of(args: Vec<Node>, prefix: Option<&str>) -> Vec<Node> {
    let mut out = Vec::new();
    if let Some(prefix) = prefix {
        out.push(text_token(prefix));
    }
    if has_op(&args, &["|", "&", "=>", "?"]) {
        out.push(Node::Group('(', args));
    } else {
        out.extend(args);
    }
    out.push(Node::Group('[', Vec::new()));
    out
}

fn python_generic_alias(name: &str) -> Option<&'static str> {
    Some(match name {
        "List" => "list",
        "Dict" => "dict",
        "Set" => "set",
        "FrozenSet" => "frozenset",
        "Tuple" => "tuple",
        "Type" => "type",
        _ => return None,
    })
}

fn csharp_keyword(metadata: &str) -> Option<&'static str> {
    Some(match metadata {
        "Boolean" => "bool",
        "Byte" => "byte",
        "Char" => "char",
        "Decimal" => "decimal",
        "Double" => "double",
        "Single" => "float",
        "Int32" => "int",
        "Int64" => "long",
        "IntPtr" => "nint",
        "UIntPtr" => "nuint",
        "Object" => "object",
        "SByte" => "sbyte",
        "Int16" => "short",
        "String" => "string",
        "UInt32" => "uint",
        "UInt64" => "ulong",
        "UInt16" => "ushort",
        "Void" => "void",
        _ => return None,
    })
}

/// Drops parameter labels of a nested function type where the profile says so:
/// each Go label, and each Dart positional label.
fn drop_labels(nodes: Vec<Node>, language: &Language) -> Vec<Node> {
    match language.name {
        "go" => {
            let is_call = |previous: Option<&Node>| {
                matches!(
                    previous,
                    Some(Node::Token(Lexeme::Word, _) | Node::Group('(', _))
                )
            };
            let mut out: Vec<Node> = Vec::new();
            for node in nodes {
                match node {
                    Node::Group('(', inner) if is_call(out.last()) => {
                        out.push(Node::Group('(', go_unlabel(inner)));
                    }
                    other => out.push(other),
                }
            }
            out
        }
        "dart" => {
            let mut out: Vec<Node> = Vec::new();
            for node in nodes {
                match node {
                    Node::Group('(', inner) if follows_function_keyword(&out) => {
                        out.push(Node::Group('(', dart_unlabel(inner)));
                    }
                    other => out.push(other),
                }
            }
            out
        }
        _ => nodes,
    }
}

/// Whether the last node is `Function`, or `Function` and its generic binders.
fn follows_function_keyword(out: &[Node]) -> bool {
    match out {
        [.., keyword, Node::Group('<', _)] => word(Some(keyword)) == Some("Function"),
        [.., keyword] => word(Some(keyword)) == Some("Function"),
        [] => false,
    }
}

/// `x int, y ...T` to `int, ...T`, and `a, b int` to `int, int`.
fn go_unlabel(inner: Vec<Node>) -> Vec<Node> {
    let items = split(&inner, ",");
    let labelled = items
        .iter()
        .any(|item| item.len() >= 2 && word(item.first()).is_some() && !go_type_start(item));
    if !labelled {
        return inner;
    }
    let mut out: Vec<Vec<Node>> = Vec::new();
    let mut pending = 0;
    for item in items {
        if item.len() == 1 {
            pending += 1;
            continue;
        }
        let ty = item[1..].to_vec();
        for _ in 0..pending {
            out.push(ty.clone());
        }
        pending = 0;
        out.push(ty);
    }
    let mut joined = Vec::new();
    for (i, item) in out.into_iter().enumerate() {
        if i > 0 {
            joined.push(op_token(","));
        }
        joined.extend(item);
    }
    joined
}

/// Whether an item begins with a Go type rather than a label.
fn go_type_start(item: &[Node]) -> bool {
    matches!(
        word(item.first()),
        Some("func" | "map" | "chan" | "struct" | "interface")
    ) || matches!(item.get(1), Some(Node::Group('[', _)))
}

fn dart_unlabel(inner: Vec<Node>) -> Vec<Node> {
    let mut out = Vec::new();
    for (i, item) in split(&inner, ",").into_iter().enumerate() {
        if i > 0 {
            out.push(op_token(","));
        }
        let mut item = item;
        match item.as_mut_slice() {
            [Node::Group('[', optional)] => {
                *optional = dart_unlabel(std::mem::take(optional));
            }
            [_, .., Node::Token(Lexeme::Word, _)] => {
                item.pop();
            }
            _ => {}
        }
        out.extend(item);
    }
    out
}

/// A TypeScript member's name, whichever way it is spelled, or empty for a
/// signature with no name. `readonly x` and `"x"` are both named `x`.
fn member_name(member: &[Node]) -> String {
    let mut nodes = member.iter().peekable();
    while let Some(node) = nodes.next() {
        match node {
            Node::Token(Lexeme::Other, text) if matches!(text.as_str(), "+" | "-") => {}
            Node::Token(Lexeme::Word, text) => {
                let modifier = matches!(text.as_str(), "readonly" | "get" | "set" | "static")
                    && matches!(
                        nodes.peek(),
                        Some(Node::Token(Lexeme::Word | Lexeme::Text | Lexeme::Number, _))
                    );
                if modifier {
                    continue;
                }
                return if text == "new" {
                    String::new()
                } else {
                    text.clone()
                };
            }
            Node::Token(Lexeme::Text, text) => {
                return serde_json::from_str(text).unwrap_or_else(|_| text.clone());
            }
            Node::Token(Lexeme::Number, text) => return text.clone(),
            _ => return String::new(),
        }
    }
    String::new()
}

/// Sorts a TypeScript type literal's named members, and a Go interface's
/// methods and then its embedded elements. A Go struct's order is its layout.
fn sort_members(
    open: char,
    inner: Vec<Node>,
    before: Option<&Node>,
    language: &Language,
) -> Vec<Node> {
    if open != '{' || inner.is_empty() {
        return inner;
    }
    match language.name {
        "typescript" => {
            let mut members: Vec<Vec<Node>> = inner
                .split(|node| is_op(node, ";") || is_op(node, ","))
                .filter(|member| !member.is_empty())
                .map(<[Node]>::to_vec)
                .collect();
            // A signature with no name keeps its order, ahead of named ones;
            // repeated names keep theirs, because the sort is stable.
            members.sort_by_key(|member| member_name(member));
            join_members(members)
        }
        "go" if word(before) == Some("interface") => {
            let members: Vec<Vec<Node>> = inner
                .split(|node| is_op(node, ";"))
                .filter(|member| !member.is_empty())
                .map(<[Node]>::to_vec)
                .collect();
            let (mut methods, mut embedded): (Vec<_>, Vec<_>) = members
                .into_iter()
                .partition(|member| matches!(member.get(1), Some(Node::Group('(', _))));
            methods.sort_by_key(|member| key(member, language));
            embedded.sort_by_key(|member| key(member, language));
            methods.extend(embedded);
            join_members(methods)
        }
        _ => inner,
    }
}

fn join_members(members: Vec<Vec<Node>>) -> Vec<Node> {
    let mut out = Vec::new();
    for (i, member) in members.into_iter().enumerate() {
        if i > 0 {
            out.push(op_token(";"));
        }
        out.extend(member);
    }
    out
}

fn normalize(nodes: Vec<Node>, language: &Language, names: &dyn NameResolver) -> Vec<Node> {
    let mut normalized: Vec<Node> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let node = match node {
            Node::Group(open, inner) => {
                let inner = normalize(inner, language, names);
                Node::Group(open, sort_members(open, inner, normalized.last(), language))
            }
            token => token,
        };
        normalized.push(node);
    }
    let nodes = rewrite(normalized, language, names);
    let nodes = drop_labels(nodes, language);
    let nodes = drop_redundant_parentheses(nodes, language);
    let nodes = flatten_unions(nodes, language);
    sort_sets(nodes, language)
}

/// `A | (B | C)` to `A | B | C`, in Python, whose union is a flat set.
fn flatten_unions(nodes: Vec<Node>, language: &Language) -> Vec<Node> {
    if language.name != "python" || !has_op(&nodes, &["|"]) {
        return nodes;
    }
    nodes
        .into_iter()
        .flat_map(|node| match node {
            Node::Group('(', inner) if has_op(&inner, &["|"]) => inner,
            other => vec![other],
        })
        .collect()
}

/// Renders nodes in the language's spacing.
fn render(nodes: &[Node], language: &Language) -> String {
    let mut flat = Vec::new();
    flatten(nodes, &mut flat);
    let mut out = String::new();
    let mut ternary = 0usize;
    for (i, piece) in flat.iter().enumerate() {
        let previous = i.checked_sub(1).map(|at| flat[at]);
        let next = flat.get(i + 1).copied();
        // A TypeScript `?` before `:` or `,` marks an optional member; any
        // other `?` is a conditional type, spaced with its `:`.
        let conditional_mark =
            language.name == "typescript" && piece.text == "?" && !next_is_optional_end(next);
        let conditional_colon = piece.text == ":" && ternary > 0;
        if conditional_mark {
            ternary += 1;
        }
        if conditional_colon {
            ternary -= 1;
        }
        if let Some(previous) = previous {
            if conditional_mark || conditional_colon || space_between(previous, *piece, language) {
                out.push(' ');
            }
        }
        out.push_str(piece.text);
    }
    out
}

fn next_is_optional_end(next: Option<Piece<'_>>) -> bool {
    next.is_some_and(|piece| matches!(piece.text, ":" | "," | ")" | ";" | "}"))
}

#[derive(Debug, Clone, Copy)]
struct Piece<'n> {
    lexeme: Lexeme,
    text: &'n str,
}

fn flatten<'n>(nodes: &'n [Node], out: &mut Vec<Piece<'n>>) {
    for node in nodes {
        match node {
            Node::Token(lexeme, text) => out.push(Piece {
                lexeme: *lexeme,
                text,
            }),
            Node::Group(open, inner) => {
                out.push(bracket(*open));
                flatten(inner, out);
                out.push(bracket(closing(*open)));
            }
        }
    }
}

fn bracket(c: char) -> Piece<'static> {
    let text = match c {
        '(' => "(",
        ')' => ")",
        '[' => "[",
        ']' => "]",
        '{' => "{",
        '}' => "}",
        '<' => "<",
        _ => ">",
    };
    Piece {
        lexeme: Lexeme::Other,
        text,
    }
}

fn wordlike(piece: Piece<'_>) -> bool {
    matches!(
        piece.lexeme,
        Lexeme::Word
            | Lexeme::Name
            | Lexeme::Unresolved
            | Lexeme::Number
            | Lexeme::Text
            | Lexeme::Char
            | Lexeme::Lifetime
    )
}

fn space_between(previous: Piece<'_>, next: Piece<'_>, language: &Language) -> bool {
    let (p, n) = (previous.text, next.text);
    // Nothing separates a bracket from its content, except spaced braces.
    if p == "{" || n == "}" {
        return !(language.tight_braces || (p == "{" && n == "}"));
    }
    if matches!(p, "(" | "[" | "<" | "::" | "...") {
        return false;
    }
    if matches!(n, ")" | "]" | ">" | "," | ";" | ":" | "::") {
        return false;
    }
    if p == "." || n == "." {
        return false;
    }
    if n == "{" {
        // Go spells the empty struct `struct{}`, which rendering alone
        // cannot see from here; `struct {` holds for a non-empty one.
        return p != "(";
    }
    match language.name {
        "rust" if p == "?" => return false,
        "typescript" | "python" | "dart" | "csharp" if n == "?" => return false,
        "go" if p == "~" => return false,
        "csharp" if n == "~" => return false,
        "rust" | "go" | "csharp" if matches!(p, "&" | "*") && language.prefix_references => {
            return false
        }
        "go" if p == "]" => return !wordlike(next) && !matches!(n, "*" | "[" | "(" | "<"),
        _ => {}
    }
    if p == "-" {
        return false;
    }
    if matches!(n, "(" | "[" | "<") {
        let spaced =
            matches!(previous.lexeme, Lexeme::Word) && language.spaced_keywords.contains(&p);
        return if wordlike(previous) && !matches!(previous.lexeme, Lexeme::Lifetime) {
            spaced
        } else if matches!(p, ")" | "]" | ">") {
            language.name == "go" && p == ")" && n == "("
        } else {
            true
        };
    }
    true
}

/// Words of a text that a rule does not account for: not predeclared, not
/// syntax, and not in a label position.
fn collect_bare(
    nodes: &[Node],
    language: &Language,
    in_struct: bool,
    binders: &mut Vec<String>,
    out: &mut Vec<String>,
) {
    for (i, node) in nodes.iter().enumerate() {
        let next = nodes.get(i + 1);
        match node {
            Node::Group(open, inner) => {
                let is_struct = *open == '{'
                    && language.name == "go"
                    && i > 0
                    && word(nodes.get(i - 1)) == Some("struct");
                let local_generics = matches!(language.name, "typescript" | "dart")
                    && *open == '<'
                    && matches!(next, Some(Node::Group('(', _)));
                if local_generics {
                    for part in split(inner, ",") {
                        if let Some(name) = word(part.first()) {
                            binders.push(name.to_owned());
                        }
                    }
                }
                collect_bare(inner, language, is_struct, binders, out);
            }
            Node::Token(Lexeme::Word, text) => {
                let previous = i.checked_sub(1).and_then(|at| nodes.get(at));
                if word(previous) == Some("infer")
                    || (language.name == "typescript" && word(next) == Some("in"))
                {
                    binders.push(text.clone());
                    continue;
                }
                if word(previous) == Some("untyped")
                    || language.predeclared.contains(&text.as_str())
                    || language.keywords.contains(&text.as_str())
                    || binders.contains(text)
                    || is_label(nodes, i, language, in_struct)
                {
                    continue;
                }
                out.push(text.clone());
            }
            _ => {}
        }
    }
}

fn is_label(nodes: &[Node], at: usize, language: &Language, in_struct: bool) -> bool {
    let next = nodes.get(at + 1);
    let followed_by = |op: &str| next.is_some_and(|node| is_op(node, op));
    match language.name {
        "rust" => followed_by("=") || (at > 0 && is_op(&nodes[at - 1], "::")) || followed_by(":"),
        "typescript" => {
            followed_by(":")
                || (followed_by("?")
                    && nodes
                        .get(at + 2)
                        .is_some_and(|node| is_op(node, ":") || is_op(node, ",")))
                || (matches!(next, Some(Node::Group('(', _))))
        }
        "go" => {
            // A struct field's or an interface method's name.
            let starts_member = at == 0 || is_op(&nodes[at - 1], ";");
            (in_struct && starts_member && !followed_by(";") && next.is_some())
                || (starts_member && matches!(next, Some(Node::Group('(', _))))
        }
        // A tuple element's name, and a named parameter's.
        "csharp" | "dart" => {
            let ends_item = next.is_none_or(|node| is_op(node, ","));
            let starts_item = at == 0 || is_op(&nodes[at - 1], ",");
            ends_item && !starts_item
        }
        _ => false,
    }
}

/// Single-quoted strings to double-quoted, in the languages that have both.
fn normalize_quotes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    let mut open: Option<char> = None;
    while let Some(c) = chars.next() {
        match (open, c) {
            // Inside a double-quoted or template literal, only its end matters.
            (Some(quote), _) => {
                out.push(c);
                if c == '\\' {
                    out.extend(chars.next());
                } else if c == quote {
                    open = None;
                }
            }
            (None, '"' | '`') => {
                open = Some(c);
                out.push(c);
            }
            (None, '\'') => {
                out.push('"');
                while let Some(inner) = chars.next() {
                    match inner {
                        '\'' => break,
                        // An escaped apostrophe needs no escape in double quotes.
                        '\\' => match chars.next() {
                            Some('\'') => out.push('\''),
                            Some(escaped) => {
                                out.push('\\');
                                out.push(escaped);
                            }
                            None => out.push('\\'),
                        },
                        '"' => out.push_str("\\\""),
                        other => out.push(other),
                    }
                }
                out.push('"');
            }
            (None, other) => out.push(other),
        }
    }
    out
}

/// Tokenizes one type text and applies its language's profile rules. An unknown
/// language returns the text unchanged, with an issue.
pub fn canonicalize(language_name: &str, text: &str, names: &dyn NameResolver) -> Canonical {
    let Some(language) = language(language_name) else {
        return Canonical {
            text: text.to_owned(),
            issues: vec![format!("no canonical rules for `{language_name}`")],
            ..Canonical::default()
        };
    };
    let source = quoted_for(language.name, text);
    let mut issues = Vec::new();
    let stream = tokens(&source);
    if stream
        .iter()
        .any(|(lexeme, _)| *lexeme == Lexeme::Unresolved)
    {
        issues.push("`@?{}` is an observation form, and a fact resolves every name".to_owned());
    }
    let nested = nest(stream, language, &mut issues);
    let nodes = normalize(nested, language, names);
    let rendered = render(&nodes, language);
    if language.name == "rust" {
        projection_issues(&nodes, &mut issues);
    }
    let mut bare = Vec::new();
    collect_bare(&nodes, language, false, &mut Vec::new(), &mut bare);
    bare.sort();
    bare.dedup();
    let ids = named_ids(language.name, &rendered);
    Canonical {
        text: rendered,
        ids,
        bare,
        issues,
    }
}

/// Canonicalizes a `bounds` slot, whose items a profile can order beyond the type
/// syntax: C#'s constraint list, a Dart mixin's `on` clause and a Python
/// `TypeVar`'s constraints.
pub fn canonicalize_bounds(language: &str, text: &str, names: &dyn NameResolver) -> Canonical {
    let whole = canonicalize(language, text, names);
    let (prefix, inner, suffix) = match language {
        "csharp" | "dart" => ("", whole.text.clone(), ""),
        "python" => match whole
            .text
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
        {
            Some(inner) => ("(", inner.to_owned(), ")"),
            None => return whole,
        },
        _ => return whole,
    };
    let mut items: Vec<String> = split_top_level(&inner)
        .into_iter()
        .map(str::to_owned)
        .collect();
    if language == "csharp" {
        items = order_csharp_constraints(items, names);
    } else {
        items.sort();
    }
    Canonical {
        text: format!("{prefix}{}{suffix}", items.join(", ")),
        ..whole
    }
}

/// The language fixes a primary constraint first, then `new()`, then
/// `allows ref struct`; the constraints between are sorted.
fn order_csharp_constraints(items: Vec<String>, names: &dyn NameResolver) -> Vec<String> {
    let is_primary = |item: &str| {
        let keyword = item.trim_end_matches('?');
        matches!(
            keyword,
            "class" | "struct" | "unmanaged" | "notnull" | "default"
        ) || item
            .strip_prefix("@{")
            .and_then(|rest| rest.split('}').next())
            .and_then(|id| names.language_kind(id))
            .is_some_and(|kind| matches!(kind, "class" | "record"))
    };
    let (mut first, mut middle, mut new, mut allows) = (vec![], vec![], vec![], vec![]);
    for item in items {
        if item == "new()" {
            new.push(item);
        } else if item.starts_with("allows ") {
            allows.push(item);
        } else if is_primary(&item) {
            first.push(item);
        } else {
            middle.push(item);
        }
    }
    middle.sort();
    first
        .into_iter()
        .chain(middle)
        .chain(new)
        .chain(allows)
        .collect()
}

/// `T::Item` for a bare parameter is written `<T as @{Tr}>::@{Tr.Item}`.
fn projection_issues(nodes: &[Node], issues: &mut Vec<String>) {
    for (i, node) in nodes.iter().enumerate() {
        if let Node::Group(_, inner) = node {
            projection_issues(inner, issues);
        }
        if let (Some(name), Some(next)) = (word(Some(node)), nodes.get(i + 1)) {
            let qualified = i > 0 && is_op(&nodes[i - 1], "::");
            if is_op(next, "::") && !qualified && name != "Self" {
                issues.push(format!("write `{name}::` as a qualified projection"));
            }
        }
    }
}

/// How many `impl` types a Rust type text holds, and whether the whole text is
/// one of them.
pub fn rust_opaque_types(text: &str) -> (usize, bool) {
    let stream = tokens(text);
    let count = stream
        .iter()
        .filter(|(lexeme, word)| *lexeme == Lexeme::Word && word == "impl")
        .count();
    let whole = stream
        .first()
        .is_some_and(|(lexeme, word)| *lexeme == Lexeme::Word && word == "impl");
    (count, whole)
}

/// A decimal integer with one spelling: no sign on zero, and no leading zero.
fn is_decimal_integer(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let spelled = digits == "0" || digits.starts_with(|c: char| ('1'..='9').contains(&c));
    spelled && digits.chars().all(|c| c.is_ascii_digit()) && text != "-0"
}

/// A decimal point form: an integer part with one spelling, and a fraction.
fn split_decimal(text: &str) -> Option<(&str, &str)> {
    let (whole, fraction) = text.strip_prefix('-').unwrap_or(text).split_once('.')?;
    let spelled = whole == "0" || (is_decimal_integer(whole) && !whole.is_empty());
    (spelled && !fraction.is_empty() && fraction.chars().all(|c| c.is_ascii_digit()))
        .then_some((whole, fraction))
}

/// The shortest decimal that round-trips, in a point or exponent form.
fn is_shortest_float(text: &str) -> bool {
    if matches!(text, "NaN" | "inf" | "-inf" | "Infinity" | "-Infinity") {
        return true;
    }
    let (significand, exponent) = match text.split_once(['e', 'E']) {
        Some((significand, exponent)) => (significand, Some(exponent)),
        None => (text, None),
    };
    if let Some(exponent) = exponent {
        let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    let fraction = match split_decimal(significand) {
        Some((_, fraction)) => fraction,
        None if exponent.is_some() && is_decimal_integer(significand) => "",
        None => return false,
    };
    // A trailing zero adds nothing, except the one in `1.0`.
    if fraction.len() > 1 && fraction.ends_with('0') {
        return false;
    }
    let Ok(value) = text.parse::<f64>() else {
        return false;
    };
    if !value.is_finite() {
        return false;
    }
    let digits_of = |written: &str| -> String {
        let digits: String = written.chars().filter(char::is_ascii_digit).collect();
        let digits = digits.trim_start_matches('0').trim_end_matches('0');
        digits.to_owned()
    };
    let shortest = format!("{:e}", value.abs());
    let mantissa = shortest.split('e').next().unwrap_or_default();
    digits_of(significand) == digits_of(mantissa)
}

/// `go/constant`'s exact string: an integer, a ratio, or `(re + imi)`.
fn is_go_exact(text: &str) -> bool {
    let rational = |part: &str| match part.split_once('/') {
        Some((top, bottom)) => {
            is_decimal_integer(top) && is_decimal_integer(bottom) && !bottom.starts_with('-')
        }
        None => is_decimal_integer(part),
    };
    match text
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix("i)"))
    {
        Some(inner) => inner
            .split_once(" + ")
            .is_some_and(|(real, imaginary)| rational(real) && rational(imaginary)),
        None => rational(text),
    }
}

/// Whether a value is one of the model's value forms for a language.
pub fn value_form_issue(language: &str, value: &str, is_default: bool) -> Option<String> {
    let enum_value = {
        let stream = tokens(value);
        stream.len() == 1 && stream[0].0 == Lexeme::Name
    };
    let string = value.starts_with('"') && serde_json::from_str::<String>(value).is_ok();
    let literal = match language {
        "python" => matches!(value, "None" | "True" | "False"),
        _ => matches!(value, "null" | "true" | "false"),
    };
    let number = match language {
        "go" => is_go_exact(value),
        // A C# `decimal` is its exact decimal, whose scale is part of the value.
        "csharp" => {
            is_decimal_integer(value) || split_decimal(value).is_some() || is_shortest_float(value)
        }
        _ => is_decimal_integer(value) || is_shortest_float(value),
    };
    let present = language == "python" && is_default && value == "present";
    if enum_value || string || literal || number || present {
        None
    } else {
        Some(format!("`{value}` is not a value form of `{language}`"))
    }
}

/// The key of an `auto-traits` element: the text before ` when `.
pub fn auto_trait_key(element: &str) -> &str {
    element.split(" when ").next().unwrap_or(element)
}

/// The canonical spelling of an `auto-traits` element: its conditions sorted
/// and each condition's bounds sorted.
pub fn canonical_auto_trait(element: &str, names: &dyn NameResolver) -> String {
    let Some((head, conditions)) = element.split_once(" when ") else {
        return element.to_owned();
    };
    let mut predicates: Vec<String> = split_top_level(conditions)
        .into_iter()
        .map(|predicate| canonicalize("rust", predicate, names).text)
        .collect();
    predicates.sort();
    format!("{head} when {}", predicates.join(", "))
}

/// Splits on commas outside brackets.
fn split_top_level(text: &str) -> Vec<&str> {
    let mut depth = 0i32;
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, c) in text.char_indices() {
        match c {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts.into_iter().map(str::trim).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Paths;
    impl NameResolver for Paths {
        fn standard_path<'r>(&'r self, id: &'r str) -> Option<&'r str> {
            match id {
                "arr" => Some("typescript/lib/lib.es5.d.ts#Array"),
                "ro" => Some("typescript/lib/lib.es5.d.ts#ReadonlyArray"),
                "opt" => Some("typing.Optional"),
                "un" => Some("typing.Union"),
                "lst" => Some("typing.List"),
                "nullable" => Some("System.Nullable`1"),
                "int32" => Some("System.Int32"),
                // Project or other-package declarations of those names.
                "my-arr" => Some("pkg/index.d.ts#Array"),
                "my-opt" => Some("pkg.Optional"),
                "my-lst" => Some("pkg.typing.List"),
                "my-nullable" => Some("Example.Nullable"),
                "my-int" => Some("Example.Int32"),
                _ => None,
            }
        }

        fn language_kind(&self, id: &str) -> Option<&str> {
            (id == "base").then_some("class")
        }
    }

    fn check(language: &str, cases: &[(&str, &str)]) {
        for (input, expected) in cases {
            let first = canonicalize(language, input, &Paths);
            assert_eq!(first.text, *expected, "{language}: {input}");
            let again = canonicalize(language, &first.text, &Paths);
            assert_eq!(again.text, *expected, "{language}: not idempotent: {input}");
        }
    }

    #[test]
    fn rust_spellings() {
        check(
            "rust",
            &[
                ("&  'a   T", "&'a T"),
                ("& mut T", "&mut T"),
                ("(u8)", "u8"),
                ("(u8,)", "(u8,)"),
                ("()", "()"),
                ("( u8 , u16 )", "(u8, u16)"),
                ("[ u8 ; 4 ]", "[u8; 4]"),
                ("*const u8", "*const u8"),
                ("impl @{B} + @{A}", "impl @{A} + @{B}"),
                ("impl @{C} + 'a", "impl 'a + @{C}"),
                ("&'a dyn @{B}", "&'a dyn @{B}"),
                ("fn(u8)->u8", "fn(u8) -> u8"),
                ("extern \"C\" fn(i32) -> i32", "extern \"C\" fn(i32) -> i32"),
                ("<T as @{Tr}>::@{Tr.Item}", "<T as @{Tr}>::@{Tr.Item}"),
                ("@{O}< u32 >", "@{O}<u32>"),
                ("T: @{B} + @{A}", "T: @{A} + @{B}"),
                ("(impl @{D}, impl @{D})", "(impl @{D}, impl @{D})"),
                ("impl @{C} + use<'a, T, N>", "impl @{C} + use<'a, T, N>"),
            ],
        );
    }

    /// A rewrite needs the standard declaration, and a same-named type keeps its id.
    #[test]
    fn a_declaration_of_a_standard_name_is_not_rewritten() {
        check(
            "typescript",
            &[
                ("@{my-arr}<string>", "@{my-arr}<string>"),
                ("@{unknown}<string>", "@{unknown}<string>"),
                ("@{my-arr}<@{arr}<string>>", "@{my-arr}<string[]>"),
            ],
        );
        check(
            "python",
            &[
                ("@{my-opt}[int]", "@{my-opt}[int]"),
                ("@{my-lst}[int]", "@{my-lst}[int]"),
                ("@{my-opt}[@{opt}[int]]", "@{my-opt}[None | int]"),
            ],
        );
        check(
            "csharp",
            &[
                ("@{my-nullable}<int>", "@{my-nullable}<int>"),
                ("@{my-int}", "@{my-int}"),
            ],
        );
        // A bare word is not a standard declaration either.
        assert_eq!(
            canonicalize("python", "Optional[int]", &Paths).text,
            "Optional[int]"
        );
        assert_eq!(
            canonicalize("python", "Optional[int]", &Paths).bare,
            ["Optional"]
        );
        assert_eq!(
            canonicalize("typescript", "Array<string>", &Paths).text,
            "Array<string>"
        );
    }

    /// Spelled either way, one method's overloads keep their order.
    #[test]
    fn typescript_members_sort_by_their_semantic_name() {
        check(
            "typescript",
            &[
                (
                    "{ f(x: string): string; \"f\"(x: number): number }",
                    "{ f(x: string): string; \"f\"(x: number): number }",
                ),
                (
                    "{ \"f\"(x: number): number; f(x: string): string }",
                    "{ \"f\"(x: number): number; f(x: string): string }",
                ),
                (
                    "{ readonly b: string; readonly a: number }",
                    "{ readonly a: number; readonly b: string }",
                ),
                (
                    "{ readonly b: string; a: number }",
                    "{ a: number; readonly b: string }",
                ),
                (
                    "{ b: string; \"a\": number }",
                    "{ \"a\": number; b: string }",
                ),
                (
                    "{ readonly: string; a: number }",
                    "{ a: number; readonly: string }",
                ),
            ],
        );
    }

    /// A struct's field order is its layout, whatever its fields look like.
    #[test]
    fn a_go_struct_keeps_its_order() {
        check(
            "go",
            &[
                ("struct { @{B}; @{A} }", "struct { @{B}; @{A} }"),
                ("struct { *@{B}; *@{A} }", "struct { *@{B}; *@{A} }"),
                (
                    "struct { @{B}; A int; @{C} }",
                    "struct { @{B}; A int; @{C} }",
                ),
                ("interface { @{B}; @{A} }", "interface { @{A}; @{B} }"),
            ],
        );
    }

    #[test]
    fn csharp_constraints_order_by_the_language_rule() {
        let order = |text: &str| canonicalize_bounds("csharp", text, &Paths).text;
        assert_eq!(
            order("class, @{IB}, @{IA}, new()"),
            "class, @{IA}, @{IB}, new()"
        );
        assert_eq!(
            order("class, @{IA}, @{IB}, new()"),
            "class, @{IA}, @{IB}, new()"
        );
        assert_eq!(order("@{IB}, T, @{IA}"), "@{IA}, @{IB}, T");
        assert_eq!(order("@{base}, @{IB}, @{IA}"), "@{base}, @{IA}, @{IB}");
        assert_eq!(
            order("struct, @{IB}, @{IA}, allows ref struct"),
            "struct, @{IA}, @{IB}, allows ref struct"
        );
        // `new()` is last whatever sorts after it, and so is `allows ref struct`.
        assert_eq!(order("zeta, new()"), "zeta, new()");
        assert_eq!(order("new(), zeta"), "zeta, new()");
        assert_eq!(
            order("allows ref struct, zeta, new()"),
            "zeta, new(), allows ref struct"
        );
        assert_eq!(
            order("@{IB}, new(), allows ref struct"),
            "@{IB}, new(), allows ref struct"
        );
    }

    #[test]
    fn dart_and_python_bound_lists_sort() {
        assert_eq!(
            canonicalize_bounds("dart", "@{B}, @{A}", &Paths).text,
            "@{A}, @{B}"
        );
        assert_eq!(
            canonicalize_bounds("python", "(str, int)", &Paths).text,
            "(int, str)"
        );
        assert_eq!(canonicalize_bounds("python", "int", &Paths).text, "int");
    }

    #[test]
    fn a_dart_generic_function_type_binds_its_parameters() {
        check(
            "dart",
            &[
                ("int Function<T>(T x)", "int Function<T>(T)"),
                ("int Function<T>(T)", "int Function<T>(T)"),
                (
                    "int Function<T>(T Function<U>(U y) x)",
                    "int Function<T>(T Function<U>(U))",
                ),
                (
                    "int Function<T>(T, {T width})",
                    "int Function<T>(T, {T width})",
                ),
            ],
        );
        let found = canonicalize("dart", "int Function<T>(T x)", &Paths);
        assert_eq!(found.bare, Vec::<String>::new(), "{found:?}");
        let found = canonicalize("dart", "T Function<U>(U)", &Paths);
        assert_eq!(found.bare, ["T"], "{found:?}");
    }

    #[test]
    fn escaped_quotes_convert_to_one_literal() {
        check(
            "typescript",
            &[
                ("'can\\'t'", "\"can't\""),
                ("'say \"hi\"'", "\"say \\\"hi\\\"\""),
                ("\"it's\"", "\"it's\""),
                ("`it's`", "`it's`"),
                ("'a' | 'b\\'c'", "\"a\" | \"b'c\""),
            ],
        );
        check("python", &[("@{X}['can\\'t']", "@{X}[\"can't\"]")]);
    }

    #[test]
    fn rust_flags_a_bare_projection() {
        let found = canonicalize("rust", "T::Item", &Paths);
        assert_eq!(found.issues.len(), 1, "{found:?}");
    }

    #[test]
    fn typescript_spellings() {
        check(
            "typescript",
            &[
                ("number|string", "number | string"),
                ("string|number", "number | string"),
                ("@{arr}<string>", "string[]"),
                ("@{ro}<string>", "readonly string[]"),
                ("@{arr}<string|number>", "(number | string)[]"),
                ("{b:string;a:number}", "{ a: number; b: string }"),
                ("{ a?: string }", "{ a?: string }"),
                ("(x:T)=>U", "(x: T) => U"),
                ("(string)", "string"),
                (
                    "{ (v: string): string; (v: number): number }",
                    "{ (v: string): string; (v: number): number }",
                ),
                ("A extends B ? C : D", "A extends B ? C : D"),
                ("'a' | 'b'", "\"a\" | \"b\""),
                ("keyof typeof @{X}", "keyof typeof @{X}"),
                ("@{X}[\"k\"]", "@{X}[\"k\"]"),
                ("(number | string)[]", "(number | string)[]"),
                ("B & A | C", "A & B | C"),
            ],
        );
    }

    #[test]
    fn go_spellings() {
        check(
            "go",
            &[
                ("interface{}", "any"),
                ("byte", "uint8"),
                ("[]rune", "[]int32"),
                ("[2]int", "[2]int"),
                ("map[string]int", "map[string]int"),
                ("*@{C}", "*@{C}"),
                ("...string", "...string"),
                ("func(x int) int", "func(int) int"),
                ("func(a, b int) (int, error)", "func(int, int) (int, error)"),
                ("struct { A int; B string }", "struct { A int; B string }"),
                ("struct { B string; A int }", "struct { B string; A int }"),
                (
                    "interface { ~string | ~int }",
                    "interface { ~int | ~string }",
                ),
                (
                    "interface { Z() int; A(x int) int }",
                    "interface { A(int) int; Z() int }",
                ),
                ("@{P}[T]", "@{P}[T]"),
                ("chan []int", "chan []int"),
                ("untyped int", "untyped int"),
                ("untyped rune", "untyped rune"),
                (
                    "struct { Label string \"json:\\\"label\\\"\" }",
                    "struct { Label string \"json:\\\"label\\\"\" }",
                ),
            ],
        );
    }

    #[test]
    fn python_spellings() {
        check(
            "python",
            &[
                ("@{opt}[int]", "None | int"),
                ("@{un}[int, str]", "int | str"),
                ("@{lst}[int]", "list[int]"),
                ("int | int", "int"),
                ("str|int", "int | str"),
                ("int | (str | bytes)", "bytes | int | str"),
                ("dict[str,int]", "dict[str, int]"),
                ("@{X}['a']", "@{X}[\"a\"]"),
                ("tuple[int, ...]", "tuple[int, ...]"),
                ("tuple[(str | int), bytes]", "tuple[int | str, bytes]"),
                ("tuple[str | int, bytes]", "tuple[int | str, bytes]"),
                ("list[(int)]", "list[int]"),
            ],
        );
    }

    #[test]
    fn csharp_spellings() {
        check(
            "csharp",
            &[
                ("@{nullable}<int>", "int?"),
                ("@{int32}", "int"),
                ("string ?", "string?"),
                ("string ~", "string~"),
                ("(int x,string? y)", "(int x, string? y)"),
                ("int []", "int[]"),
                ("char[24]", "char[24]"),
                ("@{H}?", "@{H}?"),
                ("class, @{IA}, new()", "class, @{IA}, new()"),
            ],
        );
    }

    #[test]
    fn dart_spellings() {
        check(
            "dart",
            &[
                ("T ?", "T?"),
                ("int Function(String x)", "int Function(String)"),
                (
                    "int Function(String, {int width})",
                    "int Function(String, {int width})",
                ),
                ("int Function([int x])", "int Function([int])"),
                ("@{L}< int >", "@{L}<int>"),
            ],
        );
    }

    #[test]
    fn bare_words_exclude_syntax_labels_and_predeclared_names() {
        let bare = |language: &str, text: &str| canonicalize(language, text, &Paths).bare;
        assert_eq!(bare("rust", "&'a T"), ["T"]);
        assert_eq!(bare("rust", "impl @{I}<Item = u8>"), Vec::<String>::new());
        assert_eq!(bare("rust", "Other"), ["Other"]);
        assert_eq!(bare("typescript", "{ a: string; b?: T }"), ["T"]);
        assert_eq!(bare("typescript", "<U>(x: U) => U"), Vec::<String>::new());
        assert_eq!(bare("typescript", "T extends infer R ? R : never"), ["T"]);
        assert_eq!(bare("go", "struct { Left T; Right int }"), ["T"]);
        assert_eq!(
            bare("go", "interface { M(int) string }"),
            Vec::<String>::new()
        );
        assert_eq!(bare("python", "Foo | int"), ["Foo"]);
        assert_eq!(bare("csharp", "(int x, T y)"), ["T"]);
        assert_eq!(bare("dart", "int Function(String, {T width})"), ["T"]);
    }

    #[test]
    fn named_ids_skip_literals() {
        assert_eq!(named_ids("rust", "@{a}<'\"', @{b}>"), ["a", "b"]);
        assert_eq!(named_ids("rust", "\"@{a}\" | @{b}"), ["b"]);
        assert_eq!(named_ids("rust", "&'a @{b}"), ["b"]);
        assert_eq!(named_ids("rust", "'\\u{22}' @{b}"), ["b"]);
        for language in ["typescript", "python"] {
            assert_eq!(
                named_ids(language, "'can\\'t @{ghost}' | @{b}"),
                ["b"],
                "{language}"
            );
            assert_eq!(
                named_ids(language, "\"it's @{ghost}\" | @{b}"),
                ["b"],
                "{language}"
            );
        }
        assert_eq!(named_ids("typescript", "`it's @{ghost}` | @{b}"), ["b"]);
    }

    #[test]
    fn an_observation_form_in_a_fact_is_an_issue() {
        assert_eq!(canonicalize("rust", "@?{Beta}", &Paths).issues.len(), 1);
    }

    #[test]
    fn value_forms() {
        for (language, value) in [
            ("rust", "0"),
            ("rust", "-1"),
            ("rust", "1.5"),
            ("rust", "\"a\\\"b\""),
            ("rust", "true"),
            ("rust", "@{x}"),
            ("python", "None"),
            ("python", "True"),
            ("go", "1/3"),
            ("go", "(5/2 + 3i)"),
            ("dart", "null"),
        ] {
            assert_eq!(
                value_form_issue(language, value, true),
                None,
                "{language} {value}"
            );
        }
        for (language, value) in [
            ("rust", "@{x} + 1"),
            ("rust", "01"),
            ("rust", "-0"),
            ("rust", "1.00"),
            ("rust", ".5"),
            ("rust", "1.50"),
            ("rust", "+1"),
            ("rust", "0.30000000000000004000"),
            // Parses to 0.1, whose shortest spelling is shorter.
            ("rust", "0.1000000000000000055"),
            ("go", "+"),
            ("go", "1.5"),
            ("go", "(1/2 + i)"),
            ("go", "1/"),
            ("python", "1.00"),
        ] {
            assert!(
                value_form_issue(language, value, false).is_some(),
                "{language} {value}"
            );
        }
        for (language, value) in [
            ("rust", "1.0"),
            ("rust", "0.1"),
            ("rust", "1e300"),
            ("rust", "1.5e-7"),
            ("python", "1e+300"),
            ("csharp", "1.50"),
            ("go", "(1 + -2i)"),
            ("go", "-1/3"),
        ] {
            assert_eq!(
                value_form_issue(language, value, false),
                None,
                "{language} {value}"
            );
        }
        assert!(value_form_issue("python", "present", true).is_none());
        assert!(value_form_issue("python", "present", false).is_some());
        assert!(value_form_issue("rust", "present", true).is_some());
        assert!(value_form_issue("rust", "None", true).is_some());
        assert!(value_form_issue("python", "null", true).is_some());
        assert!(value_form_issue("rust", "1 + 1", true).is_some());
    }

    #[test]
    fn auto_trait_conditions_sort() {
        assert_eq!(
            canonical_auto_trait("Send when U: Sync + Send, T: Send", &Paths),
            "Send when T: Send, U: Send + Sync"
        );
        assert_eq!(auto_trait_key("#1 Send when T: Send"), "#1 Send");
        assert_eq!(canonical_auto_trait("Unpin", &Paths), "Unpin");
    }
}
