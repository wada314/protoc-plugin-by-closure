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

mod parameter;

use ::anyhow::{Result, anyhow};
use ::ipc_channel::ipc::{IpcSender, bytes_channel};
use ::std::io::{Read, Write, stdin, stdout};

use self::parameter::{CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER, find_last_string_field};

fn main() -> Result<()> {
    let mut input_buffer = Vec::new();
    stdin().read_to_end(&mut input_buffer)?;

    let ipc_init_key = find_last_string_field(
        &input_buffer,
        CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER,
    )?
    .ok_or_else(|| {
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
