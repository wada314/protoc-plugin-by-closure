# AI References for protoc-plugin-by-closure Project

## Summary

This document serves as a comprehensive reference for AI agents and developers working on the protoc-plugin-by-closure project. It documents key architectural decisions, implementation details, and design rationale.

## Project Overview
This project provides a way to use Google Protocol Buffer compiler (`protoc`) with closure code in Rust. It implements a protoc plugin system that allows custom code generation through user-defined closures.

## Implementation Language
- **Primary implementation**: Rust
- **Focus**: Providing flexible protoc plugin functionality for Rust developers

## Architecture & Design Decisions

### Why protobuf-core Instead of a Full Protobuf Library?

**Decision**: Use `protobuf-core` crate for all protobuf parsing needs instead of comprehensive protobuf libraries like `prost`.

**Rationale**:
- **Minimal requirements**: This crate only needs to:
  - Parse one field from `CodeGeneratorRequest` (the `parameter` field containing IPC init key)
  - Serialize `CodeGeneratorResponse` messages in tests
- **Dependency footprint**: Full protobuf libraries are too large for such minimal needs
- **Maintainability**: Less code to maintain, fewer transitive dependencies

**protobuf-core capabilities used**:
- Varint encoding/decoding
- Tag operations
- Field reading/writing utilities
- Wire format constants and types

**Related Resources**: 
- `protobuf-core` crate: https://github.com/wada314/protobuf-core
- Documentation: https://docs.rs/protobuf-core/
- Version used: 0.1.0

## Project Structure

```
protoc-plugin-by-closure/
├── lib/                              # Main library crate
│   ├── src/lib.rs                    # Public API (Protoc, ProtocOnMemory)
│   ├── tests/
│   │   ├── compiler_plugin/mod.rs   # Minimal protobuf message implementations
│   │   ├── test_on_memory.rs        # Tests for on-memory execution
│   │   └── test_call_wrapper.rs     # Tests for plugin call wrapper
│   └── Cargo.toml
├── bin/                              # Plugin binary crate
│   ├── src/main.rs                   # Protobuf parsing with protobuf-core
│   └── Cargo.toml
└── AI_REFERENCES.md                  # This file
```

## Key Features
- IPC-based communication with protoc
- On-memory plugin execution support
- Flexible closure-based code generation
- Timeout support for plugin execution

## Technical Dependencies
- `ipc-channel` - For IPC communication
- `thiserror` - For error handling
- `protoc-plugin-bin` - Internal binary artifact
- `wait-timeout` - For timeout support
- `tempfile` - For on-memory feature (optional)

## Implementation Details

### bin/src/main.rs - Plugin Binary

**Purpose**: Receives `CodeGeneratorRequest` from protoc via stdin, extracts the IPC initialization key from the `parameter` field, and establishes IPC communication with the library.

**Key Implementation**:
```rust
// Field number for CodeGeneratorRequest.parameter field
// See: google/protobuf/compiler/plugin.proto
const CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER: u32 = 2;

fn find_last_string_field(input: &[u8], target_field_number: u32) -> Result<Option<String>> {
    // Uses protobuf-core to parse protobuf fields without full message deserialization
    for field_result in input.read_protobuf_fields() {
        let field = field_result?;
        if field.field_number.as_u32() == target_field_number {
            if let FieldValue::Len(bytes) = field.value {
                result = Some(String::from_utf8(bytes)?);
            }
        }
    }
    Ok(result)
}
```

**Why this approach?**
- Only one field (`parameter`) needs to be extracted
- Full message deserialization would be overkill
- Minimal dependencies and code footprint
- Uses `protobuf-core` for reliable protobuf parsing

### lib/tests/compiler_plugin/mod.rs - Test Helper

**Purpose**: Minimal implementation of `google.protobuf.compiler` messages for testing, using only protobuf-core APIs.

**Implemented Messages**:
- `CodeGeneratorRequest` - Parses and counts `proto_file` fields (field 15)
- `CodeGeneratorResponse` - Serializes list of generated files (field 15)
- `File` - Represents generated files with `name` (field 1) and `content` (field 15)

**Key Design Decisions**:
1. **Returns `protobuf_core::Result`**: Uses protobuf-core's error type instead of converting to `std::io::Error`
2. **Minimal implementation**: Only implements fields needed for testing
3. **No full deserialization**: `CodeGeneratorRequest` only counts files, doesn't parse `FileDescriptorProto`
4. **Module structure**: Placed in `tests/compiler_plugin/mod.rs` to avoid being treated as a test crate

**Example Usage**:
```rust
// In test code
let req = CodeGeneratorRequest::from_bytes(req_bytes)?;
assert_eq!(req.proto_file_count, 1);

let mut res = CodeGeneratorResponse::default();
res.files.push(File {
    name: "output.rs".to_string(),
    content: "// generated code".to_string(),
});
res.to_bytes(&mut writer)?;
```

### lib/src/lib.rs - Main Library

**Purpose**: Provides high-level API for running protoc with custom plugin code via closures.

**No protobuf parsing**: This module only manages process execution, IPC communication, and file I/O. All protobuf parsing is delegated to the plugin binary.

## For Future Developers / AI Agents

### Adding New Protobuf Functionality

If you need to extend protobuf parsing capabilities:

1. **Use protobuf-core APIs**: Don't write manual parsing code
2. **Reference field numbers**: See `google/protobuf/compiler/plugin.proto` and `google/protobuf/descriptor.proto` for official field definitions
3. **Follow existing patterns**: See `lib/tests/compiler_plugin/mod.rs` for examples

### Common Tasks

**Parsing additional fields from CodeGeneratorRequest**:
```rust
// Add constant for field number (from plugin.proto)
const MY_FIELD_NUMBER: u32 = X;

// Use the same iteration pattern as find_last_string_field()
for field_result in input.read_protobuf_fields() {
    let field = field_result?;
    if field.field_number.as_u32() == MY_FIELD_NUMBER {
        // Handle field based on its FieldValue variant
    }
}
```

**Extending test helper with new message types**:
- Add message structs to `lib/tests/compiler_plugin/mod.rs`
- Implement parsing with `ReadExtProtobuf::read_protobuf_fields()`
- Implement serialization with `WriteExtProtobuf::write_protobuf_field()`
- Return `protobuf_core::Result` for consistency

### Dependencies

**Production dependencies**:
- `protobuf-core = "0.1.0"` (in `bin/Cargo.toml` only)
- Keep this minimal - no full protobuf libraries

**Dev dependencies**:
- `protobuf-core = "0.1.0"` (in `lib/Cargo.toml`)
- Used only for test helpers

### Testing

Run tests with: `cargo test`

Tests verify:
- IPC communication between library and plugin binary
- Protobuf parsing in the plugin binary
- Both on-memory and file-based execution modes

