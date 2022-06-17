//! The flow content type.
//!
//! **Flow** represents the sections, such as headings, code, and content, which
//! is parsed per line.
//! An example is HTML, which has a certain starting condition (such as
//! `<script>` on its own line), then continues for a while, until an end
//! condition is found (such as `</style>`).
//! If that line with an end condition is never found, that flow goes until
//! the end.
//!
//! The constructs found in flow are:
//!
//! *   [Blank line][crate::construct::blank_line]
//! *   [Code (fenced)][crate::construct::code_fenced]
//! *   [Code (indented)][crate::construct::code_indented]
//! *   [Definition][crate::construct::definition]
//! *   [Heading (atx)][crate::construct::heading_atx]
//! *   [Heading (setext)][crate::construct::heading_setext]
//! *   [HTML (flow)][crate::construct::html_flow]
//! *   [Thematic break][crate::construct::thematic_break]
//!
//! <!-- To do: Link to content. -->

use crate::constant::TAB_SIZE;
use crate::construct::{
    blank_line::start as blank_line, code_fenced::start as code_fenced,
    code_indented::start as code_indented, definition::start as definition,
    heading_atx::start as heading_atx, heading_setext::start as heading_setext,
    html_flow::start as html_flow, partial_whitespace::start as whitespace,
    thematic_break::start as thematic_break,
};
use crate::subtokenize::subtokenize;
use crate::tokenizer::{Code, Event, Point, State, StateFnResult, TokenType, Tokenizer};
use crate::util::span::from_exit_event;

/// Turn `codes` as the flow content type into events.
pub fn flow(codes: &[Code], point: Point, index: usize) -> Vec<Event> {
    let mut tokenizer = Tokenizer::new(point, index);
    tokenizer.feed(codes, Box::new(start), true);
    let mut result = (tokenizer.events, false);
    while !result.1 {
        result = subtokenize(result.0, codes);
    }
    result.0
}

/// Before flow.
///
/// First we assume a blank line.
//
/// ```markdown
/// |
/// |## alpha
/// |    bravo
/// |***
/// ```
pub fn start(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        _ => tokenizer.attempt(blank_line, |ok| {
            Box::new(if ok { blank_line_after } else { initial_before })
        })(tokenizer, code),
    }
}

/// After a blank line.
///
/// Move to `start` afterwards.
///
/// ```markdown
/// ␠␠|
/// ```
fn blank_line_after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::BlankLineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::BlankLineEnding);
            (State::Fn(Box::new(start)), None)
        }
        _ => unreachable!("expected eol/eof after blank line `{:?}`", code),
    }
}

/// Before flow (initial).
///
/// “Initial” flow means unprefixed flow, so right at the start of a line.
/// Interestingly, the only flow (initial) construct is indented code.
/// Move to `before` afterwards.
///
/// ```markdown
/// |qwe
/// |    asd
/// |~~~js
/// |<div>
/// ```
fn initial_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        // To do: should all flow just start before the prefix?
        _ => tokenizer.attempt_3(code_indented, code_fenced, html_flow, |ok| {
            Box::new(if ok { after } else { before })
        })(tokenizer, code),
    }
}

/// After a flow construct.
///
/// ```markdown
/// ## alpha|
/// |
/// ~~~js
/// asd
/// ~~~|
/// ```
fn after(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None => (State::Ok, None),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (State::Fn(Box::new(start)), None)
        }
        _ => unreachable!("unexpected non-eol/eof after flow `{:?}`", code),
    }
}

/// Before flow, but not at code (indented) or code (fenced).
///
/// Compared to flow (initial), normal flow can be arbitrarily prefixed.
///
/// ```markdown
/// |qwe
/// ```
pub fn before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt(
        |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
        |_ok| Box::new(before_after_prefix),
    )(tokenizer, code)
}

/// Before flow, after potential whitespace.
///
/// ```markdown
/// |# asd
/// |***
/// ```
pub fn before_after_prefix(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_4(
        heading_atx,
        thematic_break,
        definition,
        heading_setext,
        |ok| Box::new(if ok { after } else { content_before }),
    )(tokenizer, code)
}

/// Before content.
///
/// ```markdown
/// |qwe
/// ```
///
// To do: we don’t need content anymore in `micromark-rs` it seems?
fn content_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            after(tokenizer, code)
        }
        _ => {
            tokenizer.enter(TokenType::Content);
            tokenizer.enter(TokenType::ChunkContent);
            content(tokenizer, code, tokenizer.events.len() - 1)
        }
    }
}

/// In content.
///
/// ```markdown
/// al|pha
/// ```
fn content(tokenizer: &mut Tokenizer, code: Code, previous: usize) -> StateFnResult {
    match code {
        Code::None => content_end(tokenizer, code),
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.check(continuation_construct, move |ok| {
                Box::new(move |t, c| {
                    if ok {
                        content_continue(t, c, previous)
                    } else {
                        content_end(t, c)
                    }
                })
            })(tokenizer, code)
        }
        _ => {
            tokenizer.consume(code);
            (
                State::Fn(Box::new(move |t, c| content(t, c, previous))),
                None,
            )
        }
    }
}

fn continuation_construct(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    match code {
        Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => {
            tokenizer.enter(TokenType::LineEnding);
            tokenizer.consume(code);
            tokenizer.exit(TokenType::LineEnding);
            (
                State::Fn(Box::new(continuation_construct_initial_before)),
                None,
            )
        }
        _ => unreachable!("expected eol"),
    }
}

fn continuation_construct_initial_before(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.attempt_2(code_fenced, html_flow, |ok| {
        if ok {
            Box::new(|_tokenizer, _code| (State::Nok, None))
        } else {
            Box::new(|tokenizer, code| {
                tokenizer.attempt(
                    |tokenizer, code| whitespace(tokenizer, code, TokenType::Whitespace),
                    |_ok| Box::new(continuation_construct_after_prefix),
                )(tokenizer, code)
            })
        }
    })(tokenizer, code)
}

fn continuation_construct_after_prefix(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    let tail = tokenizer.events.last();
    let mut prefix = 0;

    if let Some(event) = tail {
        if event.token_type == TokenType::Whitespace {
            let span = from_exit_event(&tokenizer.events, tokenizer.events.len() - 1);
            prefix = span.end_index - span.start_index;
        }
    }

    match code {
        // Blank lines are not allowed in content.
        Code::None | Code::CarriageReturnLineFeed | Code::Char('\n' | '\r') => (State::Nok, None),
        // To do: If code is disabled, indented lines are part of the content.
        _ if prefix >= TAB_SIZE => (State::Ok, None),
        // To do: definitions, setext headings, etc?
        _ => tokenizer.attempt_2(heading_atx, thematic_break, |ok| {
            let result = if ok {
                (State::Nok, None)
            } else {
                (State::Ok, None)
            };
            Box::new(|_t, _c| result)
        })(tokenizer, code),
    }
}

fn content_continue(tokenizer: &mut Tokenizer, code: Code, previous_index: usize) -> StateFnResult {
    tokenizer.consume(code);
    tokenizer.exit(TokenType::ChunkContent);
    tokenizer.enter(TokenType::ChunkContent);
    let next_index = tokenizer.events.len() - 1;
    tokenizer.events[previous_index].next = Some(next_index);
    tokenizer.events[next_index].previous = Some(previous_index);
    (
        State::Fn(Box::new(move |t, c| content(t, c, next_index))),
        None,
    )
}

fn content_end(tokenizer: &mut Tokenizer, code: Code) -> StateFnResult {
    tokenizer.exit(TokenType::ChunkContent);
    tokenizer.exit(TokenType::Content);
    after(tokenizer, code)
}
