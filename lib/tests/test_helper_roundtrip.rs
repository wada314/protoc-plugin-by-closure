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

//! Unit coverage for the shared `compiler_plugin` test helper (no `protoc`).

mod compiler_plugin;

use ::protobuf_core::{AsRefExtProtobuf, Field, FieldNumber, FieldValue, WriteExtProtobuf};

use self::compiler_plugin::{
    CodeGeneratorRequest, CodeGeneratorResponse, last_string_field, response_with_one_file,
};

#[test]
fn response_encodes_one_file_as_field_15_len() {
    let bytes = response_with_one_file("a.rs", "hi");
    let mut file_count = 0;
    for field in bytes.as_slice().read_protobuf_fields() {
        let field = field.unwrap();
        if field.field_number.as_u32() == 15
            && let FieldValue::Len(_) = field.value
        {
            file_count += 1;
        }
    }
    assert_eq!(file_count, 1);
}

#[test]
fn request_counts_only_proto_file_len_fields() {
    let mut bytes = Vec::new();
    for payload in [b"file-a".as_slice(), b"file-b".as_slice()] {
        let field = Field::new(
            FieldNumber::try_from(15u32).unwrap(),
            FieldValue::Len(payload),
        );
        bytes.write_protobuf_field(&field).unwrap();
    }
    // Unrelated field must not inflate the count.
    let other = Field::new(
        FieldNumber::try_from(2u32).unwrap(),
        FieldValue::Len(b"ipc-key".as_slice()),
    );
    bytes.write_protobuf_field(&other).unwrap();

    let req = CodeGeneratorRequest::from_bytes(&bytes).unwrap();
    assert_eq!(req.proto_file_count, 2);
    assert_eq!(
        last_string_field(&bytes, 2).unwrap().as_deref(),
        Some("ipc-key")
    );
}

#[test]
fn empty_response_writes_nothing() {
    let mut bytes = Vec::new();
    let n = CodeGeneratorResponse::default()
        .to_bytes(&mut bytes)
        .unwrap();
    assert_eq!(n, 0);
    assert!(bytes.is_empty());
}
