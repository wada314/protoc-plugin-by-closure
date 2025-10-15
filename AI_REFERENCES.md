# AI References for protoc-plugin-by-closure Project

## Summary

This document serves as a comprehensive reference for AI agents working on the protoc-plugin-by-closure project. It documents the current migration effort to replace the `puroro` dependency with the minimal `protobuf-core` crate.

**Key Points**:
- **Goal**: Replace `puroro` (large protobuf library) with `protobuf-core` (minimal utility library)
- **Scope**: Replace hand-written protobuf parsing code in `bin/src/main.rs` with `protobuf-core` APIs
- **Status**: Planning phase - detailed analysis and migration plan documented below
- **Impact**: Reduced dependency footprint, more maintainable code, better error handling

## Project Overview
This project provides a way to use Google Protocol Buffer compiler (`protoc`) with closure code in Rust. It implements a protoc plugin system that allows custom code generation through user-defined closures.

## Implementation Language
- **Primary implementation**: Rust
- **Focus**: Providing flexible protoc plugin functionality for Rust developers

## Current Migration Goal

### Dependency Migration: puroro → protobuf-core

**Objective**: Replace the dependency on `puroro` crate with the minimal `protobuf-core` crate.

**Rationale**:
- `puroro` is a comprehensive protobuf implementation but is too large for our needs
- `protobuf-core` is a minimal protobuf utility library that provides only the essentials:
  - Common definitions, constants, and enums
  - Varint encoding/decoding
  - Tag operations
  - Field reading/writing utilities
  - Wire format constants
- This change will reduce the dependency footprint and align with our minimal requirements

**Current State**:
- `puroro = "0.14.0"` is currently used in `dev-dependencies` (lib/Cargo.toml) for testing
- No production dependency on `puroro` exists

**Target State**:
- Replace `puroro` with `protobuf-core` in `dev-dependencies`
- Migrate test code to use `protobuf-core` APIs
- Ensure all tests continue to pass after migration

**Related Project**: 
- `protobuf-core` crate: A minimal protobuf utility library created specifically to reduce implementation barriers
- Version: 0.1.0
- Repository: https://github.com/wada314/protobuf-core
- Documentation: https://docs.rs/protobuf-core/
- See: `../protobuf-core/AI_REFERENCES.md` for detailed information about protobuf-core design and implementation

## Project Structure
- `lib/` - Main library crate (`protoc-plugin-by-closure`)
- `bin/` - Binary crate for the protoc plugin executable (`protoc-plugin-bin`)
- `tests/` - Integration tests
  - `test_on_memory.rs` - Tests for on-memory plugin execution
  - `test_call_wrapper.rs` - Tests for plugin call wrapper functionality

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

## Code Replacement Candidates

### bin/src/main.rs - Hand-written Protobuf Parsing Code

The following hand-written protobuf parsing code can be replaced with `protobuf-core` APIs:

1. **`eat_variant_i64()` function** (lines 19-40)
   - Current: Manual varint decoding implementation
   - Replace with: `protobuf_core::ReadExtVarint::read_varint()` or `protobuf_core::IteratorExtVarint::try_collect_varint()`
   - Benefits: Tested, well-documented API from protobuf-core

2. **`eat_variant_i32()` function** (lines 42-44)
   - Current: Wraps `eat_variant_i64()` with `try_into()`
   - Replace with: `protobuf_core::Varint::try_to_int32()` after reading varint
   - Benefits: Proper error handling with `ProtobufError::VarintDowncastOutOfRange`

3. **Wire type enum** (lines 47-68)
   - Current: Local `Wire` enum with `TryFrom<i32>` implementation
   - Replace with: `protobuf_core::WireType`
   - Benefits: Standard wire type definition with proper error handling

4. **`find_last_string_field()` function** (lines 46-110)
   - Current: Manual protobuf field parsing with wire type handling
   - Replace with: `protobuf_core::ReadExtProtobuf::read_protobuf_fields()` iterator
   - Benefits: Complete field parsing with all wire types supported, cleaner code

**Implementation Strategy**:
- Replace the entire manual parsing logic in `bin/src/main.rs` with `protobuf-core` APIs
- This will make the code more maintainable and reduce the risk of parsing bugs
- The binary's functionality will remain the same - parsing `CodeGeneratorRequest` to extract the IPC init key

**Example Code Transformation**:

Before (manual parsing):
```rust
fn find_last_string_field(mut input: &[u8], target_field_number: i32) -> Result<Option<String>> {
    // ... manual wire type enum definition
    // ... manual tag and field parsing with eat_variant_i64/i32
}
```

After (using protobuf-core):
```rust
use protobuf_core::{ReadExtProtobuf, FieldValue};

fn find_last_string_field(input: &[u8], target_field_number: u32) -> Result<Option<String>> {
    let mut result: Option<String> = None;
    let mut cursor = std::io::Cursor::new(input);
    
    for field in cursor.read_protobuf_fields() {
        let field = field?;
        if field.field_number().as_u32() == target_field_number {
            if let FieldValue::Len(bytes) = field.value() {
                result = Some(String::from_utf8(bytes.clone())?);
            }
        }
    }
    
    Ok(result)
}
```

Benefits:
- No need to manually handle wire types
- No need to manually parse varints and tags
- Better error handling with `ProtobufError`
- More readable and maintainable code

### lib/src/lib.rs - No Replacement Needed

The library code (`lib/src/lib.rs`) does not contain protobuf parsing logic. It only manages:
- IPC communication with the plugin binary
- `protoc` command execution
- Temporary file management for on-memory mode

No changes needed in the library code for protobuf-core integration.

## Next Actions for Migration

### Phase 1: Replace bin/src/main.rs with protobuf-core
- [ ] 1. Add `protobuf-core = "0.1.0"` to dependencies in `bin/Cargo.toml`
- [ ] 2. Add `use protobuf_core::{ReadExtProtobuf, FieldValue};` to `bin/src/main.rs`
- [ ] 3. Remove `eat_variant_i64()` function (lines 19-40)
- [ ] 4. Remove `eat_variant_i32()` function (lines 42-44)
- [ ] 5. Remove local `Wire` enum definition (lines 47-68)
- [ ] 6. Replace `find_last_string_field()` implementation with protobuf-core APIs
- [ ] 7. Update error handling to work with `protobuf_core::ProtobufError`
- [ ] 8. Run `cargo build --package protoc-plugin-bin` to verify compilation
- [ ] 9. Run integration tests: `cargo test` to verify functionality
- [ ] 10. Manual testing with actual protoc if needed

### Phase 2: Replace dev-dependencies in tests

**Current `puroro` usage in tests** (`test_on_memory.rs` and `test_call_wrapper.rs`):
- `CodeGeneratorRequest::from_bytes_iter()` - Deserialize request from bytes
- `CodeGeneratorResponse` and `File` message construction
- `Message` and `MessageView` traits for serialization/deserialization
- Field accessors: `req.proto_file()`, `file.name_mut()`, `file.content_mut()`, etc.
- `res.to_bytes()` - Serialize response to bytes

**Challenge**: `protobuf-core` provides only low-level utilities (varint, tag, field I/O), but does NOT provide `google.protobuf.compiler.*` message implementations. These tests need full message implementation.

**Options**:
1. Keep `puroro` in `dev-dependencies` for now (only used in tests, acceptable dependency)
2. Implement minimal `CodeGeneratorRequest`/`CodeGeneratorResponse` using `protobuf-core` primitives
3. Find a lighter alternative to `puroro` that only implements compiler messages
4. Generate the compiler messages using a protoc plugin (recursive dependency)

**Recommended approach**: Keep `puroro` in `dev-dependencies` for testing purposes. The main goal of replacing `puroro` in production code (bin/src/main.rs) is more important than removing it from test code.

**Action items**:
1. ✅ Document the `puroro` usage situation
2. Keep `puroro` in `dev-dependencies` for test code
3. No immediate action needed for Phase 2

### Phase 3: Documentation and Cleanup
1. Update documentation if needed
2. Update AI_REFERENCES.md with completion status

