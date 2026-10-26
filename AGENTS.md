# AGENTS.md

Instructions for coding agents working in this repository. This file is the [AGENTS.md](https://agents.md/) entry point. Crate readmes (`lib/readme.md`, `bin/readme.md`) are for humans.

## Overview

Workspace of two crates:

| Crate | Path | Role |
|---|---|---|
| `protoc-plugin-by-closure` | `lib/` | Run `protoc` with a user closure via IPC |
| `protoc-plugin-bin` | `bin/` | Internal plugin binary; not for manual use |

The library does not parse protobuf. The binary extracts `CodeGeneratorRequest.parameter` (field 2) with `protobuf-core` and then talks IPC.

## Setup

Nightly + Cargo `-Z bindeps` (artifact dependency on the plugin binary). This repo already has `.cargo/config.toml`. If this crate is added to **another** Cargo workspace, that workspace root needs:

```toml
[unstable]
bindeps = true
```

CI uses `cargo +nightly test` and installs `protoc` (see `.github/workflows/rust.yml`).

## Build, test, lint

```sh
cargo +nightly test
cargo +nightly fmt
cargo +nightly clippy --all-targets
```

`protoc` must be on `PATH` for the tests that invoke the compiler.

## Constraints

- Keep protobuf parsing on `protobuf-core` only. Do not add `prost`, `protobuf`, or similar.
- Manifests pin `protobuf-core = "0.6.0"` (crates.io). That is not the puroro workspace git submodule; do not bump casually.
- Slice parsing uses `AsRefExtProtobuf::read_protobuf_fields` (zero-copy `Field<&[u8]>`). `ReadExtProtobuf` is for `std::io::Read`.
- `bin/` is an implementation detail of the IPC helper.

## Code style

- Imports: `crate::` / `self::` for this crate; other paths start with `::`.
- Comments in English.

## Docs

Architecture notes and the `find_last_string_field` pattern: [`AI_REFERENCES.md`](AI_REFERENCES.md). New durable agent instructions belong here in `AGENTS.md`.
