// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
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
