# AI References for protoc-plugin-by-closure Project

## Summary

This document serves as a comprehensive reference for AI agents working on the protoc-plugin-by-closure project. It documents the current migration effort to replace the `puroro` dependency with the minimal `protobuf-core` crate.

**Key Points**:
- **Goal**: Replace `puroro` (large protobuf library) with `protobuf-core` (minimal utility library)
- **Scope**: Replace hand-written protobuf parsing code in `bin/src/main.rs` and `puroro` dependency in tests
- **Status**: ✅ **COMPLETED** - Both Phase 1 and Phase 2 successfully finished
- **Impact**: 
  - Removed all `puroro` dependencies (production and dev)
  - Reduced code from 133 lines to 62 lines in bin/src/main.rs (53% reduction)
  - Created minimal ~130-line test helper using protobuf-core
  - All tests passing successfully

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
- [x] 1. Add `protobuf-core = "0.1.0"` to dependencies in `bin/Cargo.toml`
- [x] 2. Add `use protobuf_core::{ReadExtProtobuf, FieldValue};` to `bin/src/main.rs`
- [x] 3. Remove `eat_variant_i64()` function (lines 19-40)
- [x] 4. Remove `eat_variant_i32()` function (lines 42-44)
- [x] 5. Remove local `Wire` enum definition (lines 47-68)
- [x] 6. Replace `find_last_string_field()` implementation with protobuf-core APIs
- [x] 7. Update error handling to work with `protobuf_core::ProtobufError`
- [x] 8. Run `cargo build --package protoc-plugin-bin` to verify compilation
- [x] 9. Run integration tests: `cargo test` to verify functionality
- [x] 10. Manual testing with actual protoc if needed

**Status**: ✅ **COMPLETED**

**Summary of Changes**:
- Replaced ~90 lines of manual protobuf parsing code with ~20 lines using protobuf-core APIs
- Removed `eat_variant_i64()`, `eat_variant_i32()`, and local `Wire` enum
- Simplified `find_last_string_field()` function significantly
- Error handling now leverages `protobuf_core::ProtobufError` (wrapped in anyhow)
- Build successful with no errors or warnings
- Code is more maintainable and less error-prone

**Key Benefits**:
1. **Code reduction**: 90 lines → 20 lines (~78% reduction)
2. **Maintainability**: No need to maintain manual varint/tag parsing logic
3. **Reliability**: Using well-tested protobuf-core library
4. **Error handling**: Better error messages from protobuf-core
5. **Future-proof**: Automatically benefits from protobuf-core improvements

### Phase 2: Replace dev-dependencies in tests

**Status**: ✅ **COMPLETED**

**Summary of Changes**:
- Created `lib/tests/compiler_plugin.rs` - Minimal implementation of protobuf compiler messages
- Implemented `CodeGeneratorRequest`, `CodeGeneratorResponse`, and `File` using protobuf-core
- Updated `test_on_memory.rs` and `test_call_wrapper.rs` to use the new implementation
- Successfully removed `puroro = "0.14.0"` from dev-dependencies
- All tests pass successfully

**Implementation Details**:

Created minimal protobuf message implementations in `lib/tests/compiler_plugin.rs`:
- `CodeGeneratorRequest` - Parses and counts proto_file fields (field 15)
- `CodeGeneratorResponse` - Serializes file list (field 15)
- `File` - Represents generated files with name (field 1) and content (field 15)

Used protobuf-core APIs:
- `ReadExtProtobuf::read_protobuf_fields()` - for parsing request
- `WriteExtProtobuf::write_protobuf_field()` - for serializing response
- `Field`, `FieldValue`, `FieldNumber` - for field manipulation

**Key Benefits**:
1. **No external dependencies**: Tests no longer depend on the large `puroro` crate
2. **Minimal implementation**: Only ~130 lines of code for necessary functionality
3. **Direct control**: Custom implementation tailored to testing needs
4. **Learning value**: Demonstrates how to use protobuf-core for real-world protobuf messages
5. **Consistency**: Both production code (bin) and test code now use protobuf-core

### Phase 3: Documentation and Cleanup
1. Update documentation if needed
2. Update AI_REFERENCES.md with completion status

