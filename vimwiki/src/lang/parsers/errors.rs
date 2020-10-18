use super::Span;
use nom::error::{ErrorKind, ParseError};
use std::{borrow::Cow, fmt};

/// Represents an encapsulated error that is encountered
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LangParserError<'a> {
    ctx: Cow<'a, str>,
    input: Span<'a>,
    next: Option<Box<Self>>,
}

impl<'a> fmt::Display for LangParserError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: Line {}, Column {}",
            self.ctx,
            self.input.line(),
            self.input.column()
        )?;
        writeln!(f, "Input: {}", &self.input.as_unsafe_remaining_str()[..100])?;

        if let Some(next) = self.next.as_ref() {
            next.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> std::error::Error for LangParserError<'a> {}

impl<'a> LangParserError<'a> {
    pub fn unsupported() -> Self {
        Self {
            ctx: Cow::from("Unsupported"),
            input: Span::from(""),
            next: None,
        }
    }

    pub fn from_ctx(input: &Span<'a>, ctx: &'static str) -> Self {
        Self {
            ctx: Cow::from(ctx),
            input: *input,
            next: None,
        }
    }
}

impl<'a> ParseError<Span<'a>> for LangParserError<'a> {
    fn from_error_kind(input: Span<'a>, kind: ErrorKind) -> Self {
        Self {
            ctx: Cow::from(kind.description().to_string()),
            input,
            next: None,
        }
    }

    fn append(input: Span<'a>, kind: ErrorKind, other: Self) -> Self {
        let mut e = Self::from_error_kind(input, kind);
        e.next = Some(Box::new(other));
        e
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self {
            ctx: Cow::from(format!("Char {}", c)),
            input,
            next: None,
        }
    }

    fn or(self, other: Self) -> Self {
        // Pick error that has progressed further
        if self.input.start_offset() > other.input.start_offset() {
            self
        } else {
            other
        }
    }

    fn add_context(input: Span<'a>, ctx: &'static str, other: Self) -> Self {
        Self {
            ctx: Cow::from(ctx),
            input,
            next: Some(Box::new(other)),
        }
    }
}
