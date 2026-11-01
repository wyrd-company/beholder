//! Source locators: the model's coordinate system over document text.
//!
//! Lines end at LF, CR LF or CR and are numbered from 1. Columns count
//! Unicode scalar values from 1. A byte order mark at the start of a
//! document is not counted. `start` is inclusive and `end` is exclusive.

use std::fmt;

use serde_json::Value;

/// A `[line, column]` position, both counted from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Reads a `[line, column]` JSON array.
    pub fn from_value(value: &Value) -> Option<Self> {
        let pair = value.as_array()?;
        match pair.as_slice() {
            [line, column] => Some(Self::new(count(line)?, count(column)?)),
            _ => None,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.line, self.column)
    }
}

/// A JSON integer as a count. JSON Schema's `integer` admits a number with
/// a zero fractional part, such as `12.0`.
fn count(value: &Value) -> Option<usize> {
    if let Some(n) = value.as_u64() {
        return usize::try_from(n).ok();
    }
    let n = value.as_f64()?;
    // Below 2^53 every integral float is exact.
    (n.fract() == 0.0 && (0.0..9_007_199_254_740_992.0).contains(&n)).then_some(n as usize)
}

/// A span of one document identified by path, coordinates and exact text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locator {
    pub path: String,
    pub start: Position,
    pub end: Position,
    pub text: String,
}

impl Locator {
    /// Reads a locator record. Returns `None` when a member is missing or
    /// has the wrong type; the dataset schema reports that.
    pub fn from_value(value: &Value) -> Option<Self> {
        Some(Self {
            path: value.get("path")?.as_str()?.to_owned(),
            start: Position::from_value(value.get("start")?)?,
            end: Position::from_value(value.get("end")?)?,
            text: value.get("text")?.as_str()?.to_owned(),
        })
    }

    /// The locator as one YAML flow mapping, the form datasets use.
    pub fn to_yaml(&self) -> String {
        format!(
            "{{path: {}, start: [{}, {}], end: [{}, {}], text: {}}}",
            yaml_scalar(&self.path),
            self.start.line,
            self.start.column,
            self.end.line,
            self.end.column,
            yaml_scalar(&self.text)
        )
    }
}

/// A YAML 1.2 scalar for `value`: plain when that reads back as the same
/// string, otherwise double-quoted with JSON escapes, which YAML accepts.
pub fn yaml_scalar(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-' | '$'))
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "y" | "n"
        );
    if plain {
        value.to_owned()
    } else {
        double_quoted(value)
    }
}

/// A YAML double-quoted scalar. Every character YAML does not allow
/// literally in a stream is escaped.
fn double_quoted(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            ' '..='~' | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'.. => {
                out.push(c)
            }
            c if (c as u32) <= 0xffff => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push_str(&format!("\\U{:08x}", c as u32)),
        }
    }
    out.push('"');
    out
}

/// Why a locator does not hold against its document text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocatorError {
    /// A position does not lie in the document.
    OutOfRange(Position),
    /// `end` precedes `start`.
    Reversed,
    /// The text at the coordinates differs from the stored text.
    TextMismatch { actual: String },
}

impl fmt::Display for LocatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(position) => write!(f, "position {position} is not in the document"),
            Self::Reversed => write!(f, "end precedes start"),
            Self::TextMismatch { actual } => {
                write!(f, "the text at these coordinates is {}", quote(actual))
            }
        }
    }
}

fn quote(text: &str) -> String {
    serde_json::to_string(text).expect("a string serializes")
}

/// One document decoded as UTF-8, indexed by the model's coordinates.
#[derive(Debug, Clone)]
pub struct SourceText {
    chars: Vec<char>,
    /// Character offset of the first character of each line.
    line_starts: Vec<usize>,
    /// Character offset just past the content of each line, before its
    /// terminator.
    line_ends: Vec<usize>,
}

impl SourceText {
    /// Decodes document bytes. Fails when they are not UTF-8.
    pub fn decode(bytes: &[u8]) -> Result<Self, std::str::Utf8Error> {
        let text = std::str::from_utf8(bytes)?;
        Ok(Self::new(text))
    }

    pub fn new(text: &str) -> Self {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let chars: Vec<char> = text.chars().collect();
        let mut line_starts = vec![0];
        let mut line_ends = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '\n' => {
                    line_ends.push(i);
                    line_starts.push(i + 1);
                }
                '\r' => {
                    line_ends.push(i);
                    if chars.get(i + 1) == Some(&'\n') {
                        i += 1;
                    }
                    line_starts.push(i + 1);
                }
                _ => {}
            }
            i += 1;
        }
        line_ends.push(chars.len());
        Self {
            chars,
            line_starts,
            line_ends,
        }
    }

    /// The number of lines. A document that ends with a terminator has a
    /// last, empty line.
    pub fn lines(&self) -> usize {
        self.line_starts.len()
    }

    /// The content of a line, without its terminator.
    pub fn line(&self, line: usize) -> Option<String> {
        let index = line.checked_sub(1)?;
        let start = *self.line_starts.get(index)?;
        Some(self.chars[start..self.line_ends[index]].iter().collect())
    }

    /// The character offset of a position, when the position lies in the
    /// document: on an existing line, at most one column past its content.
    pub fn offset(&self, position: Position) -> Option<usize> {
        let line = position.line.checked_sub(1)?;
        let column = position.column.checked_sub(1)?;
        let start = *self.line_starts.get(line)?;
        let end = self.line_ends[line];
        let offset = start.checked_add(column)?;
        (offset <= end).then_some(offset)
    }

    /// Whether a character offset has a position: it starts a line or lies
    /// at most one column past a line's content. The LF of a CR LF pair has
    /// none.
    fn representable(&self, offset: usize) -> bool {
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        offset <= self.line_ends[line]
    }

    /// The position of a character offset.
    pub fn position(&self, offset: usize) -> Position {
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        Position::new(line + 1, offset - self.line_starts[line] + 1)
    }

    /// The text between two positions.
    pub fn slice(&self, start: Position, end: Position) -> Result<String, LocatorError> {
        let from = self.offset(start).ok_or(LocatorError::OutOfRange(start))?;
        let to = self.offset(end).ok_or(LocatorError::OutOfRange(end))?;
        if to < from {
            return Err(LocatorError::Reversed);
        }
        Ok(self.chars[from..to].iter().collect())
    }

    /// Checks a locator's coordinates and text against this document.
    pub fn check(&self, locator: &Locator) -> Result<(), LocatorError> {
        let actual = self.slice(locator.start, locator.end)?;
        if actual == locator.text {
            Ok(())
        } else {
            Err(LocatorError::TextMismatch { actual })
        }
    }

    /// The start position of every occurrence of `needle`, in document
    /// order. Occurrences may overlap.
    pub fn find(&self, needle: &str) -> Vec<Position> {
        let needle: Vec<char> = needle.chars().collect();
        if needle.is_empty() || needle.len() > self.chars.len() {
            return Vec::new();
        }
        (0..=self.chars.len() - needle.len())
            .filter(|&i| self.chars[i..i + needle.len()] == needle[..])
            .filter(|&i| self.representable(i) && self.representable(i + needle.len()))
            .map(|i| self.position(i))
            .collect()
    }

    /// The locator of `text` starting at `start`.
    pub fn locator(&self, path: &str, start: Position, text: &str) -> Option<Locator> {
        let from = self.offset(start)?;
        let to = from + text.chars().count();
        if to > self.chars.len()
            || !self.representable(to)
            || self.chars[from..to].iter().copied().ne(text.chars())
        {
            return None;
        }
        Some(Locator {
            path: path.to_owned(),
            start,
            end: self.position(to),
            text: text.to_owned(),
        })
    }
}

/// Why a capture request does not name exactly one span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    /// The text does not occur where the request looks.
    NotFound,
    /// The text occurs at several of the positions the request allows.
    Ambiguous(Vec<Position>),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "the text does not occur there"),
            Self::Ambiguous(positions) => {
                let list: Vec<String> = positions.iter().map(Position::to_string).collect();
                write!(f, "the text occurs at {}", list.join(", "))
            }
        }
    }
}

/// Captures the locator of `text` in a document. `line` restricts the
/// search to occurrences that start on that line, and `column` to the one
/// that starts at that column. The capture fails unless exactly one
/// occurrence remains.
pub fn capture(
    source: &SourceText,
    path: &str,
    text: &str,
    line: Option<usize>,
    column: Option<usize>,
) -> Result<Locator, CaptureError> {
    let positions: Vec<Position> = source
        .find(text)
        .into_iter()
        .filter(|p| line.is_none_or(|line| p.line == line))
        .filter(|p| column.is_none_or(|column| p.column == column))
        .collect();
    match positions.as_slice() {
        [] => Err(CaptureError::NotFound),
        [start] => Ok(source
            .locator(path, *start, text)
            .expect("a found occurrence is a locator")),
        _ => Err(CaptureError::Ambiguous(positions)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_terminator_ends_a_line() {
        let source = SourceText::new("a\nb\r\nc\rd");
        assert_eq!(source.lines(), 4);
        assert_eq!(
            source.slice(Position::new(4, 1), Position::new(4, 2)),
            Ok("d".into())
        );
        assert_eq!(
            source.slice(Position::new(2, 1), Position::new(3, 1)),
            Ok("b\r\n".into())
        );
    }

    #[test]
    fn columns_count_scalar_values_and_skip_the_byte_order_mark() {
        let source = SourceText::decode("\u{feff}é😀 name".as_bytes()).unwrap();
        assert_eq!(source.find("name"), vec![Position::new(1, 4)]);
        let source = SourceText::new("\tx");
        assert_eq!(source.find("x"), vec![Position::new(1, 2)]);
    }

    #[test]
    fn a_position_past_the_line_is_out_of_range() {
        let source = SourceText::new("ab\ncd");
        assert!(source.offset(Position::new(1, 3)).is_some());
        assert_eq!(source.offset(Position::new(1, 4)), None);
        assert_eq!(source.offset(Position::new(3, 1)), None);
        assert_eq!(source.offset(Position::new(0, 1)), None);
    }

    #[test]
    fn check_reports_text_and_order() {
        let source = SourceText::new("let alpha = 1;");
        let mut locator = source
            .locator("a.txt", Position::new(1, 5), "alpha")
            .unwrap();
        assert_eq!(locator.end, Position::new(1, 10));
        assert_eq!(source.check(&locator), Ok(()));
        locator.text = "beta".into();
        assert_eq!(
            source.check(&locator),
            Err(LocatorError::TextMismatch {
                actual: "alpha".into()
            })
        );
        locator.end = Position::new(1, 2);
        assert_eq!(source.check(&locator), Err(LocatorError::Reversed));
    }

    #[test]
    fn capture_needs_exactly_one_occurrence() {
        let source = SourceText::new("alpha alpha\nalpha");
        assert_eq!(
            capture(&source, "a", "alpha", Some(1), None),
            Err(CaptureError::Ambiguous(vec![
                Position::new(1, 1),
                Position::new(1, 7)
            ]))
        );
        let found = capture(&source, "a", "alpha", Some(1), Some(7)).unwrap();
        assert_eq!(
            found.to_yaml(),
            "{path: a, start: [1, 7], end: [1, 12], text: alpha}"
        );
        assert_eq!(
            capture(&source, "a", "alpha", Some(2), None).unwrap().start,
            Position::new(2, 1)
        );
        assert_eq!(
            capture(&source, "a", "gamma", None, None),
            Err(CaptureError::NotFound)
        );
    }

    #[test]
    fn integral_floats_are_positions() {
        let position = Position::from_value(&serde_json::json!([1.0, 12.0]));
        assert_eq!(position, Some(Position::new(1, 12)));
        assert_eq!(Position::from_value(&serde_json::json!([1, 1e30])), None);
        assert_eq!(Position::from_value(&serde_json::json!([1, 1.5])), None);
    }

    #[test]
    fn an_overflowing_column_is_out_of_range() {
        let source = SourceText::new("a\nb");
        assert_eq!(source.offset(Position::new(2, usize::MAX)), None);
    }

    #[test]
    fn a_span_cannot_split_a_cr_lf_pair() {
        let source = SourceText::new("a\r\nb");
        assert_eq!(
            capture(&source, "x", "\n", None, None),
            Err(CaptureError::NotFound)
        );
        assert_eq!(
            capture(&source, "x", "\r", None, None),
            Err(CaptureError::NotFound)
        );
        let pair = capture(&source, "x", "\r\n", None, None).unwrap();
        assert_eq!(source.check(&pair), Ok(()));
        assert_eq!(source.locator("x", Position::new(1, 2), "\r"), None);
    }

    #[test]
    fn yaml_scalars_read_back_as_the_same_string() {
        for value in [
            "a\u{7f}b",
            "\u{80}\u{9f}",
            "tab\there",
            "q\"\\",
            "\u{feff}x",
            "😀",
            "\u{85}",
            "\u{0}",
        ] {
            let text = format!("v: {}\n", yaml_scalar(value));
            let parsed = crate::parse_yaml(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(parsed["v"], value, "{text}");
        }
    }

    #[test]
    fn yaml_scalars_quote_what_would_not_read_back() {
        assert_eq!(yaml_scalar("src/shapes.rs"), "src/shapes.rs");
        assert_eq!(yaml_scalar("true"), "\"true\"");
        assert_eq!(yaml_scalar("r#type"), "\"r#type\"");
        assert_eq!(yaml_scalar("1st"), "\"1st\"");
        assert_eq!(yaml_scalar("a: b"), "\"a: b\"");
    }
}
