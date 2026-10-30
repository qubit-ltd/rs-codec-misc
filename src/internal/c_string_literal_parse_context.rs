// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Parsing context used by the C string literal codec.

use qubit_utils::nonzero;

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
    #[must_use]
    #[inline]
    pub(crate) fn is_complete_text(self) -> bool {
        matches!(self, Self::CompleteText(_) | Self::EofBytes)
    }

    /// Tests whether parsing is for an open stream.
    #[must_use]
    #[inline]
    pub(crate) fn is_streaming(self) -> bool {
        matches!(self, Self::StreamingBytes)
    }

    /// Builds the error for a trailing escape marker.
    ///
    /// # Parameters
    ///
    /// - `marker_index` is the byte offset of the trailing escape marker.
    /// - `_available` is the visible byte count supplied by the parser; this
    ///   helper currently does not use it to choose the error.
    ///
    /// # Returns
    ///
    /// A malformed-input error for complete input, or an incomplete-input
    ///   error for an open stream.
    pub(crate) fn trailing_escape_error(self, marker_index: usize, _available: usize) -> ParseError {
        match self {
            Self::CompleteText(_) | Self::EofBytes => {
                crate::c_string_literal_codec::invalid_escape(marker_index, "\\", "incomplete escape sequence").into()
            }
            Self::StreamingBytes => ParseError::Incomplete { required: nonzero(2) },
        }
    }

    /// Gets the source character at a byte index for diagnostics.
    ///
    /// # Parameters
    ///
    /// - `input` is the byte representation of the source being parsed.
    /// - `index` is a valid byte index into `input`.
    ///
    /// # Returns
    ///
    /// The character beginning at `index`; byte contexts map that byte
    ///   directly to a character.
    #[must_use]
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
    ///
    /// # Returns
    ///
    /// A static explanation that distinguishes text input from byte input.
    #[must_use]
    #[inline]
    pub(crate) fn raw_source_reason(self) -> &'static str {
        match self {
            Self::CompleteText(_) | Self::EofBytes => {
                "raw source character must be printable ASCII or allowed whitespace"
            }
            Self::StreamingBytes => "raw source byte must be printable ASCII or allowed whitespace",
        }
    }

    /// Builds an escape fragment for diagnostics from a byte-indexed range.
    ///
    /// For complete text, the end of the range is rounded up to the next UTF-8
    /// character boundary, so the fragment remains valid text without falling
    /// back to the entire remaining suffix. Byte contexts retain their bounded
    /// byte-to-character mapping.
    ///
    /// # Parameters
    ///
    /// - `input` is the byte representation of the source being parsed.
    /// - `start` is the inclusive byte offset of the fragment.
    /// - `end` is the exclusive byte offset of the fragment.
    ///
    /// # Returns
    ///
    /// An owned diagnostic fragment. `start` must be within the source byte
    ///   slice; `end` is capped at its length, and text ranges are adjusted to
    ///   a UTF-8 character boundary at the end.
    #[must_use]
    pub(crate) fn escape_fragment(self, input: &[u8], start: usize, end: usize) -> String {
        match self {
            Self::CompleteText(text) => {
                let mut fragment_end = end.min(text.len());
                while !text.is_char_boundary(fragment_end) {
                    fragment_end += 1;
                }
                text.get(start..fragment_end).unwrap_or("\\").to_owned()
            }
            Self::EofBytes | Self::StreamingBytes => input[start..end.min(input.len())]
                .iter()
                .map(|byte| char::from(*byte))
                .collect(),
        }
    }
}
