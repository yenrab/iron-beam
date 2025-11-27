//! Binary Operations Module
//!
//! Provides binary data handling for Erlang terms.

/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 1996-2025.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

/// Binary data structure
pub struct Binary {
    data: Vec<u8>,
}

impl Binary {
    /// Create a new binary
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get binary data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_creation() {
        let data = vec![1, 2, 3, 4];
        let binary = Binary::new(data.clone());
        assert_eq!(binary.data(), &data);
    }
}

