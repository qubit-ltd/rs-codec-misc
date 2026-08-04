// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Configuration shared by hexadecimal byte codecs.

/// Configuration shared by [`crate::HexCodec`] encoding and decoding
/// operations.
///
/// The configuration is intentionally separate from the codec value so the
/// same formatting and decoding policy can be reused by multiple codecs.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexCodecConfig {
    /// Whether to use uppercase hexadecimal digits while encoding.
    pub(crate) uppercase: bool,
    /// The prefix to use before the whole encoded string.
    pub(crate) prefix: Option<String>,
    /// The prefix to use before each encoded byte.
    pub(crate) byte_prefix: Option<String>,
    /// The separator to use between bytes in the encoded string.
    pub(crate) separator: Option<String>,
    /// Whether to ignore ASCII whitespace while decoding.
    pub(crate) ignore_ascii_whitespace: bool,
    /// Whether to ignore ASCII case when matching configured prefixes.
    pub(crate) ignore_prefix_case: bool,
}

impl HexCodecConfig {
    /// Creates a lowercase configuration without prefixes or separators.
    #[inline]
    pub fn new() -> Self {
        Self {
            uppercase: false,
            prefix: None,
            byte_prefix: None,
            separator: None,
            ignore_ascii_whitespace: false,
            ignore_prefix_case: false,
        }
    }

    /// Sets whether encoded digits should be uppercase.
    #[inline]
    pub fn with_uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = uppercase;
        self
    }

    /// Sets a whole-output prefix.
    #[inline]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Sets a per-byte prefix.
    #[inline]
    pub fn with_byte_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.byte_prefix = Some(prefix.into());
        self
    }

    /// Sets a separator between encoded bytes.
    #[inline]
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = Some(separator.into());
        self
    }

    /// Sets whether ASCII whitespace is ignored while decoding.
    #[inline]
    pub fn with_ignored_ascii_whitespace(mut self, ignore: bool) -> Self {
        self.ignore_ascii_whitespace = ignore;
        self
    }

    /// Sets whether configured prefixes are matched case-insensitively.
    #[inline]
    pub fn with_ignore_prefix_case(mut self, ignore: bool) -> Self {
        self.ignore_prefix_case = ignore;
        self
    }

    /// Returns whether encoded digits use uppercase characters.
    #[must_use]
    #[inline]
    pub fn is_uppercase(&self) -> bool {
        self.uppercase
    }

    /// Returns the configured whole-output prefix.
    #[must_use]
    #[inline]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Returns the configured per-byte prefix.
    #[must_use]
    #[inline]
    pub fn byte_prefix(&self) -> Option<&str> {
        self.byte_prefix.as_deref()
    }

    /// Returns the configured separator.
    #[must_use]
    #[inline]
    pub fn separator(&self) -> Option<&str> {
        self.separator.as_deref()
    }

    /// Returns whether ASCII whitespace is ignored while decoding.
    #[must_use]
    #[inline]
    pub fn ignores_ascii_whitespace(&self) -> bool {
        self.ignore_ascii_whitespace
    }

    /// Returns whether configured prefixes are matched case-insensitively.
    #[must_use]
    #[inline]
    pub fn ignores_prefix_case(&self) -> bool {
        self.ignore_prefix_case
    }
}

impl Default for HexCodecConfig {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
