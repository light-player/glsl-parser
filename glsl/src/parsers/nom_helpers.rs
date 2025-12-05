//! Various nom parser helpers.

#[cfg(not(feature = "std"))]
use alloc::vec;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, line_ending, multispace1};
use nom::combinator::{map, recognize, value};
use nom::error::{ErrorKind, VerboseError, VerboseErrorKind};
use nom::multi::fold_many0;
use nom::{Err as NomErr, IResult};
use nom_locate::LocatedSpan;

use crate::syntax::SourceSpan;

/// Input type with location tracking
pub type Span<'a> = LocatedSpan<&'a str>;

/// Parser result with span tracking
pub type SpanResult<'a, T> = IResult<Span<'a>, T, VerboseError<Span<'a>>>;

/// Legacy parser result type (for backward compatibility during migration)
pub type ParserResult<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

/// Helper to extract SourceSpan from nom's LocatedSpan
pub fn to_source_span(span: Span, len: usize) -> SourceSpan {
  SourceSpan::new(
    span.location_offset(),
    span.location_line() as usize,
    span.get_column(),
    len,
  )
}

/// Helper to get span between two positions
pub fn span_between(start: Span, end: Span) -> SourceSpan {
  let len = end.location_offset() - start.location_offset();
  to_source_span(start, len)
}

// A constant parser that just forwards the value it's parametered with without reading anything
// from the input. Especially useful as "fallback" in an alternative parser.
pub fn cnst<'a, T, E>(t: T) -> impl FnMut(Span<'a>) -> Result<(Span<'a>, T), E>
where
  T: 'a + Clone,
{
  move |i| Ok((i, t.clone()))
}

// End-of-input parser.
//
// Yields `()` if the parser is at the end of the input; an error otherwise.
pub fn eoi(i: Span) -> SpanResult<'_, ()> {
  if i.fragment().is_empty() {
    Ok((i, ()))
  } else {
    Err(NomErr::Error(VerboseError {
      errors: vec![(i, VerboseErrorKind::Nom(ErrorKind::Eof))],
    }))
  }
}

// A newline parser that accepts:
//
// - A newline.
// - The end of input.
pub fn eol(i: Span) -> SpanResult<'_, ()> {
  alt((
    eoi, // this one goes first because it's very cheap
    value((), line_ending),
  ))(i)
}

// Apply the `f` parser until `g` succeeds. Both parsers consume the input.
pub fn till<'a, A, B, F, G>(mut f: F, mut g: G) -> impl FnMut(Span<'a>) -> SpanResult<'a, ()>
where
  F: FnMut(Span<'a>) -> SpanResult<'a, A>,
  G: FnMut(Span<'a>) -> SpanResult<'a, B>,
{
  move |mut i| loop {
    if let Ok((i2, _)) = g(i) {
      break Ok((i2, ()));
    }

    let (i2, _) = f(i)?;
    i = i2;
  }
}

// A version of many0 that discards the result of the parser, preventing allocating.
pub fn many0_<'a, A, F>(mut f: F) -> impl FnMut(Span<'a>) -> SpanResult<'a, ()>
where
  F: FnMut(Span<'a>) -> SpanResult<'a, A>,
{
  move |i| fold_many0(&mut f, || (), |_, _| ())(i)
}

/// Parse a string until the end of line.
///
/// This parser accepts the multiline annotation (\) to break the string on several lines.
///
/// Discard any leading newline.
pub fn str_till_eol(i: Span) -> SpanResult<'_, &str> {
  map(
    recognize(till(alt((value((), tag("\\\n")), value((), anychar))), eol)),
    |span: Span| {
      let fragment = span.fragment();
      if fragment.as_bytes().last() == Some(&b'\n') {
        &fragment[0..fragment.len() - 1]
      } else {
        fragment
      }
    },
  )(i)
}

// Blank base parser.
//
// This parser succeeds with multispaces and multiline annotation.
//
// Taylor Swift loves it.
pub fn blank_space(i: Span) -> SpanResult<'_, &str> {
  map(recognize(many0_(alt((multispace1, tag("\\\n"))))), |span: Span| -> &str {
    span.fragment()
  })(i)
}
