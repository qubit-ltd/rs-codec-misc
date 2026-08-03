pub(crate) fn invalid_source(
    failure: qubit_codec::DecodeFailure<qubit_codec_misc::MiscCodecError>,
) -> qubit_codec_misc::MiscCodecError {
    match failure {
        qubit_codec::DecodeFailure::Invalid { source, .. } => source,
        other => {
            panic!("expected invalid misc codec decode failure: {other:?}")
        }
    }
}

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
/// Extracts the retry size from a low-level incomplete decode failure.
pub(crate) fn incomplete_required(
    failure: qubit_codec::DecodeFailure<qubit_codec_misc::MiscCodecError>,
) -> usize {
    match failure {
        qubit_codec::DecodeFailure::Incomplete { required_total, .. } => {
            required_total.get()
        }
        other => {
            panic!("expected incomplete misc codec decode failure: {other:?}")
        }
    }
}
