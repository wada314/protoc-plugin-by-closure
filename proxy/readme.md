# protoc-plugin-proxy

This is a proxy crate that manages the dependency on `protoc-plugin-bin` binary, allowing the main library to compile with both stable and nightly Rust compilers.

## Features

- **`with-binary`**: Enables the `protoc-plugin-bin` binary dependency using the unstable `bindeps` feature (requires nightly compiler)

## Usage

This crate is used internally by `protoc-plugin-by-closure`. Users typically don't need to interact with it directly.

### For stable compiler users:
```toml
[dependencies]
protoc-plugin-by-closure = "0.1.9"
# No features needed - the library will work without the binary dependency
```

### For nightly compiler users who want the full functionality:
```toml
[dependencies]
protoc-plugin-by-closure = { version = "0.1.9", features = ["with-binary"] }
```

## API

- `get_plugin_path()`: Returns the path to the protoc-plugin-bin binary if available
- `has_binary()`: Checks if the binary dependency is available
