A library for running protoc command with your own plugin code as a closure.

When you develop your own protoc compiler plugin, normally you need to compile your code as an executable binary and run `protoc` command by yourself with passing the path to the binary.
This crate provides a convenient replacement for that process: You just need to pass your code as a closure to this library, then it will run the `protoc` command as like as your closure is an executable binary.

# Install

**Do not use a crates.io version number as the dependency.** That build fails today.

This crate ships an internal plugin binary through Cargo's unstable [`bindeps`](https://rust-lang.github.io/rfcs/3028-cargo-binary-dependencies.html) (artifact dependency). crates.io does not yet store that metadata in the index ([Cargo #12555](https://github.com/rust-lang/cargo/issues/12555)), so a registry dependency never builds `protoc-plugin-bin` and compiling this crate fails with:

```text
error: environment variable `CARGO_BIN_FILE_PROTOC_PLUGIN_BIN` not defined at compile time
```

Use a **git** or **path** dependency instead (and still follow [Requirements](#requirements)):

```toml
[dependencies]
# Works: git
protoc-plugin-by-closure = { git = "https://github.com/wada314/protoc-plugin-by-closure" }

# Works: path to this repo's library crate (`lib/`)
# protoc-plugin-by-closure = { path = "../protoc-plugin-by-closure/lib" }

# Does NOT work: crates.io / version-only dependency
# protoc-plugin-by-closure = "0.2.1"
```

| Dependency | Builds? |
|---|---|
| `{ git = "…" }` | Yes |
| `{ path = "…/lib" }` | Yes |
| `"0.2.1"` (crates.io) | No — missing `CARGO_BIN_FILE_PROTOC_PLUGIN_BIN` |

# Requirements

1. **Nightly** Rust / Cargo.
2. **`bindeps` enabled** for the crate that depends on this library (and for any [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) root that contains it).

This repository already has a [.cargo/config.toml](.cargo/config.toml) that sets:

```toml
[unstable]
bindeps = true
```

That file applies only inside this repo. If you depend on this crate from **another** project or workspace, create the same `.cargo/config.toml` under **that** project's workspace root (see [Cargo config](https://doc.rust-lang.org/cargo/reference/config.html)). You can also pass `-Z bindeps` on the command line.

`protoc` must be on `PATH` when you actually run the wrapper.

# Example

```rust
use protoc_plugin_by_closure::Protoc;
use std::time::Duration;

Protoc::new()
    .proto_file("my_protobuf_file.proto")
    .proto_file("my_protobuf_file2.proto")
    .proto_path("path/to/my/input_proto_dir/")
    .out_dir("path/to/my/output_dir/")
    .run(Duration::from_secs(3), |request_bytes| {
        // Your plugin logic here, which takes the CodeGeneratorRequest bytes
        // and returns the Result of CodeGeneratorResponse bytes.
    })
    .unwrap();

// The generated file names depend on your plugin logic and the contents of
// the input proto files, but typically they will be like this:
assert!(std::path::Path::new("path/to/my/output_dir/my_protobuf_file.rs").exists());
assert!(std::path::Path::new("path/to/my/output_dir/my_protobuf_file2.rs").exists());
```

# How it works

This crate is using the `ipc-channel` crate for the communication between the internal plugin binary and your closure code.
When running the `protoc` command, this crate generates an one-time key for the IPC channel, and pass it to the internal plugin binary as a `protoc` argument.
The internal plugin binary then creates the IPC channel with the key, and sends the whole input `CodeGeneratorRequest` bytes back to the caller process.
The generated `CodeGeneratorResponse` bytes are sent back to the internal plugin binary in the same way, and the binary outputs that bytes to the stdout.

# Features

- `on-memory`: Enabled by default.
Provides [`ProtocOnMemory`] struct which makes you to run the `protoc` command without touching the actual filesystem. Because this feature is using the `tempfile` crate, you can disable it if you don't need it.

# Documentation

Currently (Sep. 2025) because of docs.rs's limitation, you can't see the documentation of this crate on docs.rs. [issue thread](https://github.com/rust-lang/docs.rs/issues/2710)
Please see the documentation on [GitHub Pages](https://wada314.github.io/protoc-plugin-by-closure/protoc_plugin_by_closure/index.html).
