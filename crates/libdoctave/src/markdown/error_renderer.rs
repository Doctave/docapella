use std::{ops::Range, str::FromStr};

use itertools::Itertools;

use crate::{render_context::RenderContext, renderable_ast::Position};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub(crate) enum Location {
    Point(usize, usize),
}

impl Location {
    fn start_col(&self) -> usize {
        match self {
            Location::Point(_, col) => *col,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Highlight {
    pub location: Location,
    pub span: usize,
    pub msg: Option<String>,
}

#[derive(Debug, PartialEq)]
struct Window {
    start: usize,
    end: usize,
}

pub(crate) fn render(
    markdown: &str,
    msg: &str,
    highlights: Vec<Highlight>,
    ctx: &RenderContext,
) -> String {
    let mut result = format!("{}\n\n", msg);
    let lines: Vec<&str> = markdown.lines().collect();
    let windows = compute_windows(&highlights);

    let fm_offset = ctx
        .file_context
        .as_ref()
        .map(|f| f.error_lines_offset)
        .unwrap_or(0);

    for window in windows {
        let start_line = window.start.saturating_sub(1).max(1); // Ensure we start within bounds, adjusting for zero indexing
        let end_line = window.end.min(lines.len()); // Ensure we don't exceed the markdown content

        for line_number in start_line..=end_line {
            let line = format!(" {}", lines[line_number - 1]);
            result.push_str(&format!(
                "{:5} │{}\n",
                line_number + fm_offset,
                line.trim_end()
            ));

            // Process highlights for this specific line within the window
            let mut line_highlights: Vec<&Highlight> = highlights
                .iter()
                .filter(|h| match h.location {
                    Location::Point(ln, _) => ln == line_number,
                })
                .collect();

            line_highlights.sort_by(|a, b| a.location.start_col().cmp(&b.location.start_col()));

            if !line_highlights.is_empty() {
                render_point(&mut result, line_highlights);
            }
        }
    }

    result.push('\n');

    result
}

fn render_point(result: &mut String, line_highlights: Vec<&Highlight>) {
    let mut index = 0;
    // --------------------------------------------------------
    // Write arrow line
    //
    let span_locations =
        line_highlights
            .iter()
            .fold(vec![], |mut acc: Vec<(usize, usize)>, next| {
                if let Some(last) = acc.last() {
                    let current_last = last.0 + last.1;
                    let next_start = match next.location {
                        Location::Point(_, start) => start,
                    };

                    let (new_start, new_span) = if current_last > next_start {
                        let overlap = current_last - next_start;

                        if overlap < next.span {
                            let new_span = next.span - overlap;
                            let new_next = next_start + overlap;

                            (new_next, new_span)
                        } else {
                            (next_start, 1)
                        }
                    } else {
                        (next_start, next.span)
                    };

                    acc.push((new_start, new_span))
                } else {
                    let start = match next.location {
                        Location::Point(_, start) => start,
                    };
                    acc.push((start, next.span));
                }

                acc
            });

    let mut last_offset = 0;
    let mut arrows_line = String::new();
    for (start, span) in &span_locations[index..] {
        if start > &last_offset {
            arrows_line.push_str(&" ".repeat(start - last_offset));
            arrows_line.push_str(&"▲".repeat(*span));
        } else if start + span > last_offset {
            arrows_line.push_str(&"▲".repeat(start + span - last_offset));
        }
        last_offset = start + span;
    }
    result.push_str(&" ".repeat(7));
    result.push_str(&arrows_line);
    result.push('\n');

    // --------------------------------------------------------
    // If more than 1 highlight on the line, draw divider
    //
    if line_highlights.len() > 1 {
        let mut last_offset = 0;
        let mut divider_line = String::new();
        for (index, highlight) in line_highlights[index..]
            .iter()
            .enumerate()
            .unique_by(|(_, h)| &h.location)
        {
            divider_line.push_str(&" ".repeat(highlight.location.start_col() - last_offset));
            if index == 0 {
                divider_line.push('│');
            } else {
                divider_line.push('╵');
            }
            last_offset = highlight.location.start_col() + 1;
        }

        result.push_str(&" ".repeat(7));
        result.push_str(&divider_line);
        result.push('\n');
    }

    while index < line_highlights.len() {
        // Write the message for the highlight

        let current_highlight = line_highlights[index];
        let mut msg_line = String::new();

        if let Some(message) = &current_highlight.msg {
            msg_line.push_str(&" ".repeat(current_highlight.location.start_col()));

            msg_line.push_str("└─ ");
            msg_line.push_str(message);

            result.push_str(&" ".repeat(7));
            result.push_str(&msg_line);
            result.push('\n');
        }

        // If not last, write a in-between padding line
        if index < line_highlights.len() - 1 {
            let mut last_offset = 0;
            let mut padding_line = String::new();

            for highlight in &line_highlights[index + 1..] {
                padding_line.push_str(&" ".repeat(highlight.location.start_col() - last_offset));
                padding_line.push('╷');
                last_offset = highlight.location.start_col() + 1;
            }

            result.push_str(&" ".repeat(7));
            result.push_str(&padding_line);
            result.push('\n');
        }

        index += 1;
    }
}

pub fn offset_attribute_error_pos(
    input: &str,
    key: &str,
    expr: &str,
    node_pos: &Position,
) -> Position {
    //
    //   foo={ .. }  (expression)
    //   foo=" .. "  (literal double quote)
    //   foo=' .. '  (literal single quote)
    //
    let expression_start_offset = input[node_pos.start.byte_offset..]
        .find(&format!("{}={{", key))
        .or(input[node_pos.start.byte_offset..].find(&format!("{}=\"", key)))
        .or(input[node_pos.start.byte_offset..].find(&format!("{}=\'", key)))
        .map(|offset| offset + key.len() + 1)
        .unwrap_or(0);

    let mut pos = node_pos.clone();

    pos.start.col += expression_start_offset;
    pos.start.byte_offset += expression_start_offset;

    pos.end.col = pos.start.col + expr.len();
    pos.end.byte_offset = pos.start.byte_offset + expr.len();

    pos
}

pub fn offset_attribute_key_error_pos(input: &str, key: &str, node_pos: &Position) -> Position {
    //
    //  Targets the key part of the attribute
    //
    //    foo (expression key without value)
    //    foo={ .. }  (expression)
    //    foo=" .. "  (literal double quote)
    //    foo=' .. '  (literal single quote)
    //
    let expression_start_offset = input[node_pos.start.byte_offset..]
        .find(&format!("{}={{", key))
        .or(input[node_pos.start.byte_offset..].find(&format!("{}=\"", key)))
        .or(input[node_pos.start.byte_offset..].find(&format!("{}=\'", key)))
        .or(input[node_pos.start.byte_offset..].find(&format!("{} ", key)))
        .or(input[node_pos.start.byte_offset..].find(&format!("{}>", key)))
        .unwrap_or(0);

    let mut pos = node_pos.clone();

    pos.start.col += expression_start_offset;
    pos.start.byte_offset += expression_start_offset;

    pos.end.col = pos.start.col + key.len();
    pos.end.byte_offset = pos.start.byte_offset + key.len();

    pos
}

fn compute_windows(highlights: &[Highlight]) -> Vec<Window> {
    let mut windows = Vec::new();

    if highlights.is_empty() {
        return windows;
    }

    // Step 1: Sort highlights by their starting line
    let mut sorted_highlights = highlights.to_vec();
    sorted_highlights.sort_by(|a, b| match (&a.location, &b.location) {
        (Location::Point(a_line, _), Location::Point(b_line, _)) => a_line.cmp(b_line),
    });

    // Initialize the first window based on the first highlight's location
    let first_highlight = &sorted_highlights[0];
    let (first_start, first_end) = match first_highlight.location {
        Location::Point(line, _) => (line, line),
    };
    let mut current_window = Window {
        start: first_start,
        end: first_end,
    };

    // Step 2: Group highlights into windows, adjusting for spans
    for highlight in sorted_highlights.iter().skip(1) {
        let (start, end) = match highlight.location {
            Location::Point(line, _) => (line, line),
        };

        if start <= current_window.end + 2 {
            // Extend the current window if the highlight is within 2 lines of the window
            current_window.end = current_window.end.max(end);
        } else {
            // Finalize the current window and start a new one if the highlight is farther away
            windows.push(current_window);
            current_window = Window { start, end };
        }
    }

    // Don't forget to add the last window
    windows.push(current_window);

    windows
}

pub fn parse_int_in_range<T: FromStr + PartialOrd>(input: &str, range: Range<T>) -> Option<T> {
    match input.parse::<T>() {
        Err(_) => None,
        Ok(i) => {
            if range.contains(&i) {
                Some(i)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixed_highlights() {
        let highlights = vec![
            Highlight {
                location: Location::Point(2, 0),
                msg: Some("Point highlight".to_string()),
                span: 1,
            },
            Highlight {
                location: Location::Point(6, 0),
                msg: Some("Span highlight".to_string()),
                span: 1,
            },
        ];
        let expected_windows = vec![Window { start: 2, end: 2 }, Window { start: 6, end: 6 }];
        assert_eq!(compute_windows(&highlights), expected_windows);
    }

    #[test]
    fn test_separated_highlights_including_span() {
        let highlights = vec![
            Highlight {
                location: Location::Point(1, 0),
                msg: Some("First span".to_string()),
                span: 1,
            },
            Highlight {
                location: Location::Point(100, 0),
                msg: Some("Far away point".to_string()),
                span: 1,
            },
        ];
        let expected_windows = vec![
            Window { start: 1, end: 1 },
            Window {
                start: 100,
                end: 100,
            },
        ];
        assert_eq!(compute_windows(&highlights), expected_windows);
    }
}
