use markdown_rs::message::Place;
use regex::Regex;

use crate::{render_context::RenderContext, Point};

use super::error_renderer::{self, Highlight, Location};

const UNEXPECTED_SELF_CLOSING_SLASH_ID: &str = "unexpected-self-closing-slash";
const UNEXPECTED_SLASH_ID: &str = "unexpected-closing-slash";
const UNEXPECTED_LAZY_ID: &str = "unexpected-lazy";
const END_TAG_MISMATCH_ID: &str = "end-tag-mismatch";
const UNEXPECTED_CHARACTER_ID: &str = "unexpected-character";
const UNEXPECTED_ATTRIBUTE_ID: &str = "unexpected-attribute";

lazy_static! {
    static ref UNEXPECTED_CLOSING_TAG: Regex = {
        // https://regex101.com/r/6fTNNL/2
        let pattern = r"Unexpected closing tag `[^`]+`, expected corresponding closing tag for `[^`]+` \((?P<starting_tag_row>\d+):(?P<starting_tag_col>\d+)";

        Regex::new(pattern).unwrap()
    };

    static ref EXPECTED_CLOSING_TAG: Regex = {
        // https://regex101.com/r/qDLEJP/2
        let pattern = r"Expected a closing tag for `[^`]+` \((?P<starting_tag_row>\d+):(?P<starting_tag_col>\d+)\)";

        Regex::new(pattern).unwrap()
    };

    static ref UNEXPECTED_CLOSING_TAG_ATTRIBUTE: Regex = {
        // https://regex101.com/r/uUI9Jf/1
        let pattern = r"(?P<row>\d+):(?P<col>\d+): (?P<msg>Unexpected attribute in closing tag, expected the end of the tag)";

        Regex::new(pattern).unwrap()
    };

    static ref END_TAG_MISMATCH: Regex = {
        // https://regex101.com/r/UhAwBg/2
        let pattern = r"Expected the closing tag `[^`]+` either before the start of `(?P<interleaved_tag>[^`]+)` \((?P<interleaved_node_line>\d+):(?P<interleaved_node_col>\d+)\), or another opening tag after that start";

        Regex::new(pattern).unwrap()
    };

    static ref UNEXPECTED_TAG_NAME_CHAR: Regex = {
        // https://regex101.com/r/Ih5Y7o/1
        let pattern = r"(?P<row>\d+):(?P<col>\d+): (?P<msg>Unexpected character `!` \(U\+0021\) before name, expected a character that can start a name, such as a letter, `\$`, or `_`)";

        Regex::new(pattern).unwrap()
    };
}

/// Takes an error message from a library we don't own, and converts
/// it into a much nicer error message that we can display.
///
/// See convo: https://github.com/wooorm/markdown-rs/issues/108
pub(crate) fn pretty_error_msg(
    msg: &markdown_rs::message::Message,
    input: &str,
    ctx: &RenderContext,
) -> String {
    if msg.rule_id.as_str() == UNEXPECTED_SLASH_ID {
        match msg.place.as_deref() {
            Some(Place::Point(p)) => {
                let line = p.line;
                let col = p.column;
                let msg = &msg.reason;

                error_renderer::render(
                    input,
                    msg,
                    vec![Highlight {
                        span: 1,
                        location: Location::Point(line, col),
                        msg: Some("Closing slash in tag".to_string()),
                    }],
                    ctx,
                )
            }
            _ => unreachable!("Guaranteed to be a point place"),
        }
    } else if msg.rule_id.as_str() == UNEXPECTED_SELF_CLOSING_SLASH_ID {
        match msg.place.as_deref() {
            Some(Place::Point(p)) => {
                let line = p.line;
                let col = p.column;
                let msg = &msg.reason;

                error_renderer::render(
                    input,
                    msg,
                    vec![Highlight {
                        span: 1,
                        location: Location::Point(line, col),
                        msg: Some("Unexpected self-closing slash".to_string()),
                    }],
                    ctx,
                )
            }
            _ => unreachable!("Guaranteed to be a point place"),
        }
    } else if msg.rule_id.as_str() == UNEXPECTED_ATTRIBUTE_ID {
        match msg.place.as_deref() {
            Some(Place::Point(p)) => {
                let line = p.line;
                let col = p.column;
                let msg = &msg.reason;

                error_renderer::render(
                    input,
                    msg,
                    vec![Highlight {
                        span: 1,
                        location: Location::Point(line, col),
                        msg: Some("Unexpected attribute".to_string()),
                    }],
                    ctx,
                )
            }
            _ => unreachable!("Guaranteed to be a point place"),
        }
    } else if msg.rule_id.as_str() == END_TAG_MISMATCH_ID {
        if let Some(captures) = END_TAG_MISMATCH.captures(&msg.reason) {
            let interleaved_node_start =
                captures["interleaved_node_line"].parse::<usize>().unwrap();
            let interleaved_node_col = captures["interleaved_node_col"].parse::<usize>().unwrap();

            match msg.place.as_deref() {
                Some(Place::Point(p)) => {
                    let start = p.line;
                    let col = p.column;
                    let msg = &msg.reason;

                    error_renderer::render(
                        input,
                        &remove_position_info(msg),
                        vec![
                            Highlight {
                                span: 1,
                                location: Location::Point(start, col),
                                msg: Some("Closing tag".to_string()),
                            },
                            Highlight {
                                span: 1,
                                location: Location::Point(
                                    interleaved_node_start,
                                    interleaved_node_col,
                                ),
                                msg: Some("Opened tag".to_string()),
                            },
                        ],
                        ctx,
                    )
                }
                _ => unreachable!("Guaranteed to be a point place"),
            }
        } else if let Some(captures) = UNEXPECTED_CLOSING_TAG.captures(&msg.reason) {
            let starting_tag_row = captures["starting_tag_row"].parse::<usize>().unwrap();
            let starting_tag_col = captures["starting_tag_col"].parse::<usize>().unwrap();

            match msg.place.as_deref() {
                Some(Place::Position(p)) => {
                    let start = p.start.line;
                    let col = p.start.column;
                    let msg = &remove_position_info(&msg.reason);

                    error_renderer::render(
                        input,
                        msg,
                        vec![
                            Highlight {
                                span: 1,
                                location: Location::Point(start, col),
                                msg: Some("Expected close tag".to_string()),
                            },
                            Highlight {
                                span: 1,
                                location: Location::Point(starting_tag_row, starting_tag_col),
                                msg: Some("Opening tag".to_string()),
                            },
                        ],
                        ctx,
                    )
                }
                _ => unreachable!("Guaranteed to be a position place"),
            }
        } else if let Some(captures) = EXPECTED_CLOSING_TAG.captures(&msg.reason) {
            let starting_tag_row = captures["starting_tag_row"].parse::<usize>().unwrap();
            let starting_tag_col = captures["starting_tag_col"].parse::<usize>().unwrap();

            match msg.place.as_deref() {
                Some(Place::Point(p)) => {
                    let start = p.line;
                    let col = p.column;
                    let msg = &remove_position_info(&msg.reason);

                    error_renderer::render(
                        input,
                        msg,
                        vec![
                            Highlight {
                                span: 1,
                                location: Location::Point(start, col),
                                msg: Some("Expected close tag".to_string()),
                            },
                            Highlight {
                                span: 1,
                                location: Location::Point(starting_tag_row, starting_tag_col),
                                msg: Some("Opening tag".to_string()),
                            },
                        ],
                        ctx,
                    )
                }
                _ => unreachable!("Guaranteed to be a point place"),
            }
        } else {
            msg.reason.clone()
        }
    } else if msg.rule_id.as_str() == UNEXPECTED_CHARACTER_ID {
        match msg.place.as_deref() {
            Some(Place::Point(p)) => {
                let line = p.line;
                let col = p.column;
                let msg = &msg
                    .reason
                    .trim_end_matches(" (note: to create a comment in MDX, use `{/* text */}`)");

                error_renderer::render(
                    input,
                    msg,
                    vec![Highlight {
                        span: 1,
                        location: Location::Point(line, col),
                        msg: Some("Unexpected character".to_string()),
                    }],
                    ctx,
                )
            }
            _ => unreachable!("Guaranteed to be a point place"),
        }
    } else if msg.rule_id.as_str() == UNEXPECTED_LAZY_ID {
        match msg.place.as_deref() {
            Some(Place::Point(p)) => {
                let line = p.line;
                let col = p.column;

                let msg = if msg.reason.contains("expression") {
                    "Unexpected start of line in expression inside container.\nExpected each line of the container be prefixed with `>` when inside a block quote, whitespace when inside a list, etc."
                } else {
                    "Unexpected start of line for component inside container.\nExpected each line of the container be prefixed with `>` when inside a block quote, whitespace when inside a list, etc."
                };

                error_renderer::render(
                    input,
                    msg,
                    vec![Highlight {
                        span: 1,
                        location: Location::Point(line, col),
                        msg: Some("Unexpected start of line".to_string()),
                    }],
                    ctx,
                )
            }
            _ => unreachable!("Guaranteed to be a point place"),
        }
    } else {
        msg.reason.clone()
    }
}

pub(crate) fn parse_position(
    msg: markdown_rs::message::Message,
    markdown: &str,
) -> Option<Box<markdown_rs::message::Place>> {
    if let Some(captures) = EXPECTED_CLOSING_TAG.captures(&msg.reason) {
        let starting_tag_row = captures["starting_tag_row"].parse::<usize>().unwrap();
        let starting_tag_col = captures["starting_tag_col"].parse::<usize>().unwrap();

        Option::Some(Box::new(markdown_rs::message::Place::Position(
            markdown_rs::unist::Position {
                start: markdown_rs::unist::Point {
                    line: starting_tag_row,
                    column: starting_tag_col,
                    offset: Point::byte_offset_from_for_and_col(
                        markdown,
                        starting_tag_row,
                        starting_tag_col,
                    ),
                },
                end: markdown_rs::unist::Point {
                    line: starting_tag_row,
                    column: starting_tag_col,
                    offset: Point::byte_offset_from_for_and_col(
                        markdown,
                        starting_tag_row,
                        starting_tag_col,
                    ),
                },
            },
        )))
    } else if msg.rule_id.as_str() == UNEXPECTED_SLASH_ID {
        // In this case, we should get the position of a slash:
        //
        //  </Box>
        //   ^
        // What we want to do is convert this into a position that
        // spans the whole tag:
        //
        //  </Box>
        //  ^^^^^^
        msg.place.as_ref().map(|place| {
            match &**place {
                Place::Point(point) => {
                    let mut start = point.clone();
                    let mut end = start.clone();
                    if let Some(offset) = markdown.chars().skip(start.offset).position(|c| c == '>')
                    {
                        // Move the column back by 1 to account for the `/`
                        start.column -= 1;
                        start.offset -= 1;

                        // Set the end position based on the determined offset
                        end.offset += offset + 1;
                        end.column += offset + 1;

                        Box::new(markdown_rs::message::Place::Position(
                            markdown_rs::unist::Position { start, end },
                        ))
                    } else {
                        Box::new(markdown_rs::message::Place::Point(start))
                    }
                }
                Place::Position(position) => {
                    // This should not occur unless we've bumped the version
                    // of markdown-rs. Handle this gracefully for now.
                    Box::new(markdown_rs::message::Place::Position(position.clone()))
                }
            }
        })
    } else {
        msg.place
    }
}

fn remove_position_info(s: &str) -> String {
    let re = Regex::new(r"\s*\(\d+:\d+\)\s*").unwrap();
    re.replace_all(s, "").trim().to_string()
}
