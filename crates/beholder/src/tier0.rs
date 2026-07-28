//! Tier 0 — indentation density.
//!
//! No grammar required. Applies to every file the walk visits, including
//! formats with no tree-sitter support.
//!
//! ## Attribution
//!
//! The indentation parser and the standard scorer in this module are vendored
//! from [`thoughtbot/complexity`](https://github.com/thoughtbot/complexity)
//! (`src/parser.rs`, `src/scoring.rs`, `src/scoring/standard.rs`).
//!
//! ```text
//! Copyright (c) 2020 Josh Clayton and thoughtbot, inc.
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in
//! all copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//! FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
//! IN THE SOFTWARE.
//! ```

/// Leading-whitespace width of every non-blank line, in order.
///
/// Vendored from `thoughtbot/complexity` `src/parser.rs`.
pub fn indentation_widths(input: &str) -> Vec<usize> {
    if input.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut whitespace = 0;
    let mut space = true;

    for c in input.chars() {
        if (c == ' ' || c == '\t') && space {
            whitespace += 1;
        } else if c == '\n' {
            if !space {
                result.push(whitespace);
            }
            whitespace = 0;
            space = true;
        } else {
            space = false;
        }
    }

    if !space {
        result.push(whitespace);
    }

    result
}

/// Standard indentation-density score for a file's contents.
///
/// Vendored from `thoughtbot/complexity` `src/scoring.rs` and
/// `src/scoring/standard.rs`, collapsed from the `ScoreVisitor` trait into a
/// single function. The `ScoreVisitor` abstraction is over indentation
/// transitions and does not generalize to the tree-sitter tiers, so beholder
/// keeps the algorithm and drops the indirection.
pub fn density(input: &str) -> f32 {
    score_widths(&indentation_widths(input))
}

const BASE_VALUE: f32 = 1.0;
const MULTIPLIER: f32 = 0.5;
const MINIMUM_LINE_LENGTH_OF_FILE: usize = 2;

fn score_widths(widths: &[usize]) -> f32 {
    let line_length = widths.len();

    if line_length <= MINIMUM_LINE_LENGTH_OF_FILE {
        return 0.0;
    }

    let mut acc = 0.0f32;
    let mut previous_score = 0.0f32;
    let mut previous_width: Option<usize> = None;

    for current in widths {
        match previous_width {
            // first line
            None => acc += BASE_VALUE,
            Some(previous) if previous < *current => {
                // indent
                acc += BASE_VALUE;
                previous_score = BASE_VALUE;
            }
            Some(previous) if previous == *current => {
                // same
                let new_score = previous_score * MULTIPLIER;
                acc += new_score;
                previous_score = new_score;
            }
            // dedent contributes nothing
            Some(_) => {}
        }

        previous_width = Some(*current);
    }

    acc.powf(2.0) / line_length as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.0001
    }

    #[test]
    fn counts_whitespace_correctly() {
        assert_eq!(indentation_widths("    def full_name"), vec![4]);
        assert_eq!(indentation_widths("class Person; end"), vec![0]);
        assert!(indentation_widths("").is_empty());
    }

    #[test]
    fn parses_a_file() {
        assert_eq!(
            indentation_widths("\n\nclass Person;end\n\nclass Dog;end\n\n"),
            vec![0, 0]
        );
    }

    #[test]
    fn empty_file_scores_zero() {
        assert!(close(score_widths(&[]), 0.0));
    }

    #[test]
    fn files_at_or_below_the_minimum_length_score_zero() {
        // Upstream `complexity` applies the same floor: the acc is computed but
        // the score is suppressed for files of two lines or fewer.
        assert!(close(score_widths(&[0, 2]), 0.0));
    }

    #[test]
    fn simple_case_matches_upstream() {
        assert!(close(score_widths(&[0, 2, 0]), 1.33333));
    }

    #[test]
    fn complex_case_matches_upstream() {
        let widths = [
            0, // base score
            2, // + base score
            2, // + base score * multiplier
            2, // + (base score * multiplier ^ 2)
            4, // + base score
            4, // + base score * multiplier
            2, // + 0
            0, // + 0
        ];

        assert!(close(score_widths(&widths), 2.2578));
    }

    #[test]
    fn density_applies_to_any_text() {
        // A file with no tree-sitter grammar still scores.
        let yaml = "root:\n  child:\n    leaf: 1\n  other: 2\n";
        assert!(density(yaml) > 0.0);
    }
}
