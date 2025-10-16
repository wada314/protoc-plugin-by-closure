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

use protoc_plugin_by_closure::Protoc;
use std::io::Write;
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};

use compiler_plugin::{CodeGeneratorRequest, CodeGeneratorResponse, File};

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
            Ok(test_call_wrapper_inner(
                req,
                out_file_name,
                out_file_content,
            ))
        })
        .unwrap();

    let actual_out = ::std::fs::read_to_string(out_dir.path().join(out_file_name)).unwrap();
    assert_eq!(actual_out, out_file_content);
}

fn test_call_wrapper_inner(
    req_bytes: &[u8],
    out_file_name: &str,
    out_file_content: &str,
) -> Vec<u8> {
    let req = CodeGeneratorRequest::from_bytes(req_bytes).unwrap();
    // Check that we received one proto file
    assert_eq!(req.proto_file_count, 1);
    
    // Create response with one file
    let file = File {
        name: out_file_name.to_string(),
        content: out_file_content.to_string(),
    };
    
    let mut res = CodeGeneratorResponse::default();
    res.files.push(file);
    
    let mut res_bytes = Vec::new();
    res.to_bytes(&mut res_bytes).unwrap();
    res_bytes
}
