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

use ::anyhow::{Result, anyhow};
use ::ipc_channel::ipc::{IpcSender, bytes_channel};
use ::protobuf_core::{FieldValue, ReadExtProtobuf};
use ::std::io::{Read, Write, stdin, stdout};

// Field number for CodeGeneratorRequest.parameter field
// See: google/protobuf/compiler/plugin.proto in the Google Protobuf repository.
//
// The parameter field contains the IPC initialization key passed via --rust-ppbc_opt.
// We use protobuf-core to parse this single field without deserializing the entire message.
const CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER: u32 = 2;

fn find_last_string_field(input: &[u8], target_field_number: u32) -> Result<Option<String>> {
    let mut result: Option<String> = None;

    for field_result in input.read_protobuf_fields() {
        let field = field_result.map_err(|e| anyhow!("Failed to parse protobuf field: {}", e))?;

        // Extract string field if it matches the target field number
        if field.field_number.as_u32() == target_field_number {
            if let FieldValue::Len(bytes) = field.value {
                result = Some(String::from_utf8(bytes)?);
            }
        }
    }

    Ok(result)
}

fn main() -> Result<()> {
    let mut input_buffer = Vec::new();
    stdin().read_to_end(&mut input_buffer)?;

    let ipc_init_key = find_last_string_field(&input_buffer, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER)?.ok_or_else(|| {
        anyhow!(
            "input CodeGeneratorRequest proto does not contain a parameter field (2) (IPC init key)."
        )
    })?;
    let ipc_init_send = IpcSender::connect(ipc_init_key)?;
    let (req_send, req_recv) = bytes_channel()?;
    let (res_send, res_recv) = bytes_channel()?;
    ipc_init_send.send((req_recv, res_send))?;

    req_send.send(&input_buffer)?;
    let response = res_recv.recv()?;

    stdout().write_all(&response)?;

    Ok(())
}
