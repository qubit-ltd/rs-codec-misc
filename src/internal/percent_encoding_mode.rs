// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private percent encoding policy.

/// Selects the unescaped character set and space handling for percent output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PercentEncodingMode {
    /// RFC 3986 unreserved characters.
    Rfc3986,
    /// WHATWG `application/x-www-form-urlencoded` characters.
    FormUrlencoded,
}

impl PercentEncodingMode {
    /// Tests whether a byte may be emitted literally.
    #[inline(always)]
    pub(crate) fn is_unescaped(self, byte: u8) -> bool {
        match self {
            Self::Rfc3986 => matches!(
                byte,
                b'A'..=b'Z'
                    | b'a'..=b'z'
                    | b'0'..=b'9'
                    | b'-'
                    | b'.'
                    | b'_'
                    | b'~'
            ),
            Self::FormUrlencoded => matches!(
                byte,
                b'A'..=b'Z'
                    | b'a'..=b'z'
                    | b'0'..=b'9'
                    | b'*'
                    | b'-'
                    | b'.'
                    | b'_'
            ),
        }
    }

    /// Tests whether a byte is encoded as a form plus sign.
    #[inline(always)]
    pub(crate) fn is_space_plus(self, byte: u8) -> bool {
        matches!(self, Self::FormUrlencoded) && byte == b' '
    }
}
