// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private parser support.

mod c_string_literal_parse_context;
mod literal_components;
mod parse_error;
mod percent_encoding_mode;

pub(crate) use c_string_literal_parse_context::CStringLiteralParseContext;
pub(crate) use literal_components::LiteralComponents;
pub(crate) use parse_error::ParseError;
pub(crate) use percent_encoding_mode::PercentEncodingMode;
