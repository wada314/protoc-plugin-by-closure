# AI References for protoc-plugin-by-closure Project

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
- Repository: https://crates.io/crates/protobuf-core (assumed)
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

## Next Actions for Migration
1. Add `protobuf-core` to `dev-dependencies` in `lib/Cargo.toml`
2. Review test files (`test_on_memory.rs`, `test_call_wrapper.rs`) to identify `puroro` usage
3. Replace `puroro` API calls with equivalent `protobuf-core` APIs
4. Run tests to verify functionality
5. Remove `puroro` from `dev-dependencies`
6. Update documentation if needed

