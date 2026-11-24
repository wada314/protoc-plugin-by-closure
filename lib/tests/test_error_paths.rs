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

//! Negative-path coverage for [`protoc_plugin_by_closure::ErrorKind`].

#[allow(dead_code)] // Shared helper also defines request parsing used by other tests.
mod compiler_plugin;

use ::protoc_plugin_by_closure::{ErrorKind, Protoc};
use ::std::io::Write;
use ::std::time::Duration;
use ::tempfile::{NamedTempFile, tempdir};

use self::compiler_plugin::{CodeGeneratorResponse, File};

#[test]
fn callback_error_is_returned() {
    let out_dir = tempdir().unwrap();
    let proto_dir = tempdir().unwrap();
    let proto_file = NamedTempFile::new_in(proto_dir.path()).unwrap();
    proto_file
        .as_file()
        .write_all(
            br#"
syntax = "proto3";
package empty;
"#,
        )
        .unwrap();

    let err = Protoc::new()
        .protoc_path("protoc")
        .out_dir(out_dir.path().to_str().unwrap())
        .proto_file(proto_file.path().to_str().unwrap())
        .proto_path(proto_dir.path().to_str().unwrap())
        .run(Duration::from_secs(3), |_req| Err("plugin failed".into()))
        .unwrap_err();

    match err {
        ErrorKind::CallbackError(msg) => assert_eq!(msg, "plugin failed"),
        other => panic!("expected CallbackError, got {other:?}"),
    }
}

#[test]
fn bad_out_dir_yields_protoc_process_error() {
    // Use a regular file as `--rust-ppbc_out` so protoc starts the plugin (IPC
    // completes) then exits non-zero when it cannot write outputs.
    let not_a_dir = NamedTempFile::new().unwrap();
    let proto_dir = tempdir().unwrap();
    let proto_file = NamedTempFile::new_in(proto_dir.path()).unwrap();
    proto_file
        .as_file()
        .write_all(
            br#"
syntax = "proto3";
package empty;
"#,
        )
        .unwrap();

    let err = Protoc::new()
        .protoc_path("protoc")
        .out_dir(not_a_dir.path().to_str().unwrap())
        .proto_file(proto_file.path().to_str().unwrap())
        .proto_path(proto_dir.path().to_str().unwrap())
        .run(Duration::from_secs(3), |_req| {
            let file = File {
                name: "out.rs".into(),
                content: "x".into(),
            };
            let mut res = CodeGeneratorResponse::default();
            res.files.push(file);
            let mut res_bytes = Vec::new();
            res.to_bytes(&mut res_bytes).unwrap();
            Ok(res_bytes)
        })
        .unwrap_err();

    match err {
        ErrorKind::ProtocProcessError(status) => assert!(!status.success()),
        other => panic!("expected ProtocProcessError, got {other:?}"),
    }
}
