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

mod compiler_plugin;

use ::protoc_plugin_by_closure::Protoc;
use ::std::io::Write;
use ::std::time::Duration;
use ::tempfile::{NamedTempFile, tempdir};

use self::compiler_plugin::{
    assert_one_proto_file_and_respond, request_parameter, response_with_files,
};

#[test]
fn test_call_wrapper() {
    let out_dir = tempdir().unwrap();
    let out_file_name = "empty_test.rs";
    let out_file_content = "This\nis\na\ntest";
    let proto_dir = tempdir().unwrap();
    let proto_file = NamedTempFile::new_in(proto_dir.path()).unwrap();
    let proto_file_content = r#"
syntax = "proto3";
package empty;
"#;

    proto_file
        .as_file()
        .write_all(proto_file_content.as_bytes())
        .unwrap();

    Protoc::new()
        .protoc_path("protoc")
        .out_dir(out_dir.path().to_str().unwrap())
        .proto_file(proto_file.path().to_str().unwrap())
        .proto_path(proto_dir.path().to_str().unwrap())
        .run(Duration::from_secs(3), |req| {
            // `--rust-ppbc_opt` must land in CodeGeneratorRequest.parameter.
            let param = request_parameter(req).unwrap();
            assert!(
                param.as_ref().is_some_and(|p| !p.is_empty()),
                "expected non-empty parameter (IPC key), got {param:?}"
            );
            Ok(assert_one_proto_file_and_respond(
                req,
                out_file_name,
                out_file_content,
            ))
        })
        .unwrap();

    let actual_out = ::std::fs::read_to_string(out_dir.path().join(out_file_name)).unwrap();
    assert_eq!(actual_out, out_file_content);
}

#[test]
fn multi_proto_files_reach_the_closure() {
    let out_dir = tempdir().unwrap();
    let proto_dir = tempdir().unwrap();
    let a = NamedTempFile::new_in(proto_dir.path()).unwrap();
    let b = NamedTempFile::new_in(proto_dir.path()).unwrap();
    a.as_file()
        .write_all(br#"syntax = "proto3"; package a;"#)
        .unwrap();
    b.as_file()
        .write_all(br#"syntax = "proto3"; package b;"#)
        .unwrap();

    Protoc::new()
        .protoc_path("protoc")
        .out_dir(out_dir.path().to_str().unwrap())
        .proto_file(a.path().to_str().unwrap())
        .proto_file(b.path().to_str().unwrap())
        .proto_path(proto_dir.path().to_str().unwrap())
        .run(Duration::from_secs(3), |req| {
            let parsed = compiler_plugin::CodeGeneratorRequest::from_bytes(req).unwrap();
            assert_eq!(parsed.proto_file_count, 2);
            Ok(response_with_files(&[("a.rs", "A"), ("b.rs", "B")]))
        })
        .unwrap();

    assert_eq!(
        ::std::fs::read_to_string(out_dir.path().join("a.rs")).unwrap(),
        "A"
    );
    assert_eq!(
        ::std::fs::read_to_string(out_dir.path().join("b.rs")).unwrap(),
        "B"
    );
}
