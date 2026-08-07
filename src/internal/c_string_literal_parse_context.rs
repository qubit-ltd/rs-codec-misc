// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Parsing context used by the C string literal codec.

use crate::internal::ParseError;

/// Parsing context for one C string literal unit.
#[derive(Debug, Clone, Copy)]
pub(crate) enum CStringLiteralParseContext<'a> {
    /// Parsing a complete UTF-8 literal fragment.
    CompleteText(&'a str),
    /// Parsing one byte value after EOF has been confirmed.
    EofBytes,
    /// Parsing one byte unit for a streaming codec caller.
    StreamingBytes,
}

impl CStringLiteralParseContext<'_> {
    /// Tests whether parsing is for a complete text fragment.
    #[inline(always)]
    pub(crate) fn is_complete_text(self) -> bool {
        matches!(self, Self::CompleteText(_) | Self::EofBytes)
    }

    /// Tests whether parsing is for an open stream.
    #[inline(always)]
    pub(crate) fn is_streaming(self) -> bool {
        matches!(self, Self::StreamingBytes)
    }

    /// Builds the error for a trailing escape marker.
    pub(crate) fn trailing_escape_error(
        self,
        marker_index: usize,
        _available: usize,
    ) -> ParseError {
        match self {
            Self::CompleteText(_) | Self::EofBytes => {
                crate::c_string_literal_codec::invalid_escape(
                    marker_index,
                    "\\",
                    "incomplete escape sequence",
                )
                .into()
            }
            Self::StreamingBytes => ParseError::Incomplete {
                required: qubit_utils::nonzero(2),
            },
        }
    }

    /// Gets the source character at a byte index for diagnostics.
    pub(crate) fn source_character(self, input: &[u8], index: usize) -> char {
        match self {
            Self::CompleteText(text) => text
                .get(index..)
                .and_then(|rest| rest.chars().next())
                .unwrap_or(char::from(input[index])),
            Self::EofBytes | Self::StreamingBytes => char::from(input[index]),
        }
    }

    /// Builds a raw source character rejection reason.
    #[inline(always)]
    pub(crate) fn raw_source_reason(self) -> &'static str {
        match self {
            Self::CompleteText(_) | Self::EofBytes => {
                "raw source character must be printable ASCII or allowed whitespace"
            }
            Self::StreamingBytes => {
                "raw source byte must be printable ASCII or allowed whitespace"
            }
        }
    }

    /// Builds an escape fragment for diagnostics.
    pub(crate) fn escape_fragment(
        self,
        input: &[u8],
        start: usize,
        end: usize,
    ) -> String {
        match self {
            Self::CompleteText(text) => text
                .get(start..end)
                .or(text.get(start..))
                .unwrap_or("\\")
                .to_owned(),
            Self::EofBytes | Self::StreamingBytes => input
                [start..end.min(input.len())]
                .iter()
                .map(|byte| char::from(*byte))
                .collect(),
        }
    }
}
