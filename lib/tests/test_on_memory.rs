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

#![cfg(feature = "on-memory")]

mod compiler_plugin;

use ::protoc_plugin_by_closure::ProtocOnMemory;
use ::std::time::Duration;

use self::compiler_plugin::{
    CodeGeneratorRequest, assert_one_proto_file_and_respond, response_with_files,
};

#[test]
fn test_on_memory() {
    let out_file_name = "empty_test.rs";
    let out_file_content = "This\nis\na\ntest";
    let proto_file_name = "empty.proto";
    let proto_file_content = r#"
syntax = "proto3";
package empty;
"#;

    let result_files = ProtocOnMemory::new()
        .add_file(proto_file_name, proto_file_content)
        .run(Duration::from_secs(3), |req| {
            Ok(assert_one_proto_file_and_respond(
                req,
                out_file_name,
                out_file_content,
            ))
        })
        .unwrap();

    assert_eq!(result_files.len(), 1);
    let (result_name, result_content) = &result_files[0];
    assert_eq!(result_name, out_file_name);
    assert_eq!(result_content, out_file_content);
}

/// `ProtocOnMemory`-specific: nested virtual input paths + multi-file round-trip.
#[test]
fn on_memory_nested_path_and_multi_file() {
    let results = ProtocOnMemory::new()
        .add_file(
            "dir/a.proto",
            r#"
syntax = "proto3";
package a;
"#,
        )
        .add_file(
            "dir/b.proto",
            r#"
syntax = "proto3";
package b;
"#,
        )
        .run(Duration::from_secs(3), |req| {
            let parsed = CodeGeneratorRequest::from_bytes(req).unwrap();
            assert_eq!(parsed.proto_file_count, 2);
            // Flat output names: `ProtocOnMemory` collects only the out_dir top level.
            Ok(response_with_files(&[("a.rs", "A"), ("b.rs", "B")]))
        })
        .unwrap();

    assert_eq!(results.len(), 2);
    // read_dir order is not guaranteed
    let mut sorted = results;
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        sorted,
        vec![("a.rs".into(), "A".into()), ("b.rs".into(), "B".into()),]
    );
}
