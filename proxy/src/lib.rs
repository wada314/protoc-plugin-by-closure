// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Proxy crate for managing protoc-plugin-bin dependency
//!
//! This crate provides access to the protoc-plugin-bin binary using the unstable `bindeps` feature.
//! This crate requires nightly compiler.

use std::sync::LazyLock;

/// Get the plugin binary path
///
/// Returns the path to the protoc-plugin-bin binary.
pub fn get_plugin_path() -> &'static str {
    static path: LazyLock<String> =
        LazyLock::new(|| std::env::var("CARGO_BIN_FILE_PROTOC_PLUGIN_BIN").unwrap());
    &path
}
