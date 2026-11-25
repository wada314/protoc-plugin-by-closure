# protoc-plugin-by-closure

Run the Google Protocol Buffer compiler (`protoc`) with your own plugin code as a Rust closure.

Normally a `protoc` plugin must be a separate executable on disk. This crate runs `protoc` for you and talks to an internal plugin binary over IPC, so you can pass plugin logic as a closure in the same process that starts the compiler.

This repository is a Cargo workspace:

| Crate | Path | Role |
|---|---|---|
| [`protoc-plugin-by-closure`](lib/) | `lib/` | Public library (use this) |
| `protoc-plugin-bin` | `bin/` | Internal plugin binary; not for manual use |

API docs: [GitHub Pages](https://wada314.github.io/protoc-plugin-by-closure/protoc_plugin_by_closure/index.html).  
The crate-level readme used by rustdoc / crates.io is [`lib/readme.md`](lib/readme.md) (kept in sync with the sections below).

## Install

Do not depend on a crates.io version number alone. That build fails today.

This crate ships an internal plugin binary through Cargo's unstable [`bindeps`](https://rust-lang.github.io/rfcs/3028-cargo-binary-dependencies.html) (artifact dependency). crates.io does not yet store that metadata in the index ([Cargo #12555](https://github.com/rust-lang/cargo/issues/12555)), so a registry dependency never builds `protoc-plugin-bin` and compiling this crate fails with:

```text
error: environment variable `CARGO_BIN_FILE_PROTOC_PLUGIN_BIN` not defined at compile time
```

Copy the two snippets below into **your** project (not this repo). Use Nightly Cargo (`rustup default nightly` or `cargo +nightly …`). `protoc` must be on `PATH` when you run the wrapper.

**1. `Cargo.toml`**

```toml
[dependencies]
protoc-plugin-by-closure = { git = "https://github.com/wada314/protoc-plugin-by-closure" }
```

**2. `.cargo/config.toml`** (project or [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) root — see [Cargo config](https://doc.rust-lang.org/cargo/reference/config.html))

```toml
[unstable]
bindeps = true
```

Without that config file you can instead pass `-Z bindeps` on every Cargo command.

### Alternatives

Local checkout of this repo (the library crate lives in `lib/`):

```toml
[dependencies]
protoc-plugin-by-closure = { path = "../protoc-plugin-by-closure/lib" }
```

(You still need the same `.cargo/config.toml` / `-Z bindeps` and Nightly.)

| Dependency | Builds? |
|---|---|
| git (snippet above) | Yes |
| `{ path = "…/lib" }` | Yes |
| `"0.2.2"` (crates.io) | No — missing `CARGO_BIN_FILE_PROTOC_PLUGIN_BIN` |

## Requirements

Same as the Install snippets: Nightly, `bindeps` enabled for the depending project, and `protoc` on `PATH` at runtime. This repository's own [`.cargo/config.toml`](.cargo/config.toml) only covers builds inside this repo.

## Example

Disk-based API shape (paths are illustrative). Output file names are whatever your closure puts in the `CodeGeneratorResponse`. For a runnable end-to-end sample, see [`ProtocOnMemory`](https://wada314.github.io/protoc-plugin-by-closure/protoc_plugin_by_closure/struct.ProtocOnMemory.html).

```rust,no_run
use protoc_plugin_by_closure::Protoc;
use std::time::Duration;

Protoc::new()
    .proto_file("my_protobuf_file.proto")
    .proto_file("my_protobuf_file2.proto")
    .proto_path("path/to/my/input_proto_dir/")
    .out_dir("path/to/my/output_dir/")
    .run(Duration::from_secs(3), |_request_bytes| {
        // Empty response → no output files. Replace with encoded
        // CodeGeneratorResponse bytes when you want files written.
        Ok(Vec::new())
    })
    .unwrap();
```

## How it works

This crate uses the `ipc-channel` crate for communication between the internal plugin binary and your closure.

When running `protoc`, the library generates a one-time key for the IPC channel and passes it to the internal plugin binary as a `protoc` argument. The binary opens the channel with that key and sends the full `CodeGeneratorRequest` bytes back to the caller. Your closure returns `CodeGeneratorResponse` bytes the same way; the binary writes them to stdout for `protoc`.

## Features

- `on-memory` (default): provides [`ProtocOnMemory`](https://wada314.github.io/protoc-plugin-by-closure/protoc_plugin_by_closure/struct.ProtocOnMemory.html), which runs `protoc` without touching the real filesystem (via `tempfile`). Disable the feature if you do not need that.

## Documentation

docs.rs cannot build this crate today because of bindeps ([docs.rs #2710](https://github.com/rust-lang/docs.rs/issues/2710)). Use the [GitHub Pages rustdoc](https://wada314.github.io/protoc-plugin-by-closure/protoc_plugin_by_closure/index.html) instead.

## License

Apache-2.0
