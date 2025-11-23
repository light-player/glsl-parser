# Plan: `no_std` Support for GLSL Parser

## Overview

This document outlines the work required to make the GLSL parser (`glsl` crate) compatible with `no_std` environments. The parser currently relies on several standard library features that need to be replaced with `no_std` alternatives.

**Assumption**: Using `alloc` is acceptable. The parser will require an allocator to be available in `no_std` environments, which is a reasonable requirement for most embedded and `no_std` use cases.

## Goals

- Enable `no_std` compilation for the core parser functionality
- Get a working proof of concept for `no_std` environments
- **Note**: Backward compatibility and migration concerns are documented separately in `migration.md`

## Current `std` Dependencies

### 1. Dependencies

#### `nom` with `std` feature

- **Location**: `glsl/Cargo.toml:21`
- **Current**: `nom = { version = "7", default-features = false, features = ["std"] }`
- **Impact**: High - nom is the core parsing library
- **Solution**:
  - Remove `features = ["std"]` from nom dependency
  - Use `nom`'s `alloc` feature (requires allocator, which is acceptable)

### 2. String Allocations

#### `ParseError::info: String`

- **Location**: `glsl/src/parser.rs:23`
- **Current**: Uses `String` for error messages
- **Impact**: High - error handling is core functionality
- **Solution**:
  - Use `alloc::string::String` directly (requires allocator, which is acceptable)
  - **Breaking Change**: Type changes from `std::string::String` to `alloc::string::String`

#### String conversions in parsers

- **Locations**:
  - `glsl/src/parsers.rs:73` - `String::from` for identifiers
  - `glsl/src/parsers.rs:402, 424` - `to_owned()` for float parsing
  - `glsl/src/parsers.rs:453, 461` - `to_owned()` for path literals
  - `glsl/src/parsers.rs:1658, 1677, 1701, 1730, 1793` - `String::from` / `to_owned()` for preprocessor directives
- **Impact**: High - many parser functions allocate strings
- **Solution**:
  - Use `alloc::string::String` (requires allocator, which is acceptable)
  - Replace `String::from` with `alloc::string::String::from`
  - Replace `to_owned()` with `alloc::string::String::from`

### 3. Collections (`Vec`)

#### `NonEmpty<T>` wrapper around `Vec<T>`

- **Location**: `glsl/src/syntax.rs:27`
- **Current**: `pub struct NonEmpty<T>(pub Vec<T>)`
- **Impact**: High - used throughout AST nodes
- **Solution**:
  - Use `alloc::vec::Vec` directly (requires allocator, which is acceptable)
  - **Breaking Change**: Type changes from `std::vec::Vec` to `alloc::vec::Vec`

#### `Vec` usage in AST nodes

- **Locations**: Throughout `glsl/src/syntax.rs`
  - `Subroutine(Vec<TypeName>)`
  - `FunCall(FunIdentifier, Vec<Expr>)`
  - `parameters: Vec<FunctionParameterDeclaration>`
  - `tail: Vec<SingleDeclarationNoType>`
  - `statement_list: Vec<Statement>`
  - Many more...
- **Impact**: Critical - AST structure depends on collections
- **Solution**: Use `alloc::vec::Vec` (requires allocator, which is acceptable)

### 4. Error Traits

#### `std::error::Error` implementation

- **Location**: `glsl/src/parser.rs:26`
- **Current**: `impl std::error::Error for ParseError {}`
- **Impact**: Medium - affects error handling ergonomics
- **Solution**:
  - Use `alloc::error::Error` (available in Rust 1.73+)
  - Conditionally implement based on features for older Rust versions

#### `std::fmt::Display` and `std::fmt::Debug`

- **Location**: `glsl/src/parser.rs:28-32`, `glsl/src/syntax.rs:21`
- **Current**: Uses `std::fmt` for formatting
- **Impact**: Medium - affects display/debug output
- **Solution**:
  - Use `core::fmt` instead (available in `no_std`)
  - `Display` and `Debug` traits are available in `core`

### 5. Parsing Utilities

#### `std::num::ParseIntError`

- **Location**: `glsl/src/parsers.rs:19, 274, 284, 291, 314`
- **Current**: Uses `std::num::ParseIntError` for integer parsing
- **Impact**: Low - only used in error handling
- **Solution**:
  - Use `core::num::ParseIntError` (available in `no_std`)
  - Or define custom error type

#### `String::parse()` and `str::parse()`

- **Locations**:
  - `glsl/src/parsers.rs:278, 286, 292, 405, 407, 426, 428`
- **Current**: Uses `parse()` for number conversions
- **Impact**: Low - parsing is fine, but error types may need adjustment
- **Solution**: Works fine with `core::str::parse()`

### 6. Transpiler Module

#### `std::fmt::Write`

- **Location**: `glsl/src/transpiler/glsl.rs:34`
- **Current**: Uses `std::fmt::Write` for GLSL output
- **Impact**: Medium - affects transpiler functionality
- **Solution**:
  - Use `core::fmt::Write` (available in `no_std`)
  - Or make transpiler optional behind feature flag

#### String building in transpiler

- **Location**: `glsl/src/transpiler/glsl.rs` (throughout)
- **Current**: Uses `String` for building output
- **Impact**: Medium - affects transpiler functionality
- **Solution**:
  - Use `alloc::string::String` (requires allocator, which is acceptable)
  - Transpiler already accepts `&mut dyn Write`, so this is fine

### 7. Visitor Module

#### `std::iter::FromIterator`

- **Location**: `glsl/src/visitor.rs:1423`, `glsl/src/syntax.rs:22`
- **Current**: Uses `std::iter::FromIterator`
- **Impact**: Low - only used in some convenience methods
- **Solution**:
  - Use `core::iter::FromIterator` (available in `no_std`)
  - Or conditionally implement based on features

### 8. Iterator Traits

#### `std::iter::{once, FromIterator}`

- **Location**: `glsl/src/syntax.rs:22`
- **Current**: Uses `std::iter` utilities
- **Impact**: Low - convenience functions
- **Solution**:
  - Use `core::iter` equivalents (available in `no_std`)

## Implementation Strategy

**Assumption**: Using `alloc` is acceptable. The parser will require an allocator to be available in `no_std` environments.

### Implementation Steps

1. **Update Cargo.toml**:

   ```toml
   [package]
   # ... existing package metadata ...
   rust-version = "1.73"  # Minimum version for alloc::error::Error

   [features]
   default = ["std"]
   std = ["nom/std"]
   # When std is disabled, nom uses alloc feature automatically

   [dependencies]
   nom = { version = "7", default-features = false, features = ["alloc"] }
   ```

2. **Replace `std` imports with conditional imports**:

   - `std::fmt` → `core::fmt` (always available)
   - `std::error::Error` → `alloc::error::Error` (Rust 1.73+)
   - `std::vec::Vec` → `alloc::vec::Vec` (direct replacement)
   - `std::string::String` → `alloc::string::String` (direct replacement)
   - `std::iter` → `core::iter` (always available)
   - `std::num::ParseIntError` → `core::num::ParseIntError` (always available)

3. **Update `ParseError`**:

   ```rust
   use alloc::string::String;

   pub struct ParseError {
       pub info: String,  // Now alloc::string::String
   }
   ```

4. **Update AST types**:

   - Change `NonEmpty<T>` to use `alloc::vec::Vec<T>` directly
   - Change all `Vec<T>` fields to `alloc::vec::Vec<T>` directly
   - Update imports in `syntax.rs` to use `alloc::vec::Vec`

5. **Update parser functions**:

   - Replace `String::from` → `alloc::string::String::from`
   - Replace `to_owned()` → `alloc::string::String::from`
   - Update all string operations to use `alloc::string::String`

6. **Update error trait implementation**:

   ```rust
   #[cfg(feature = "std")]
   impl std::error::Error for ParseError {}

   #[cfg(not(feature = "std"))]
   impl alloc::error::Error for ParseError {}  // Rust 1.73+ (minimum version)
   ```

7. **Add `#![no_std]` attribute**:

   ```rust
   #![cfg_attr(not(feature = "std"), no_std)]
   #![cfg_attr(not(feature = "std"), feature(extern_crate_alloc))]
   extern crate alloc;
   ```

### Feature Flags

```toml
[package]
rust-version = "1.73"  # Minimum version for alloc::error::Error

[features]
default = ["std"]
std = ["nom/std"]  # Enable std features for nom
# When std is disabled, nom uses alloc feature (specified in dependencies)

[dependencies]
nom = { version = "7", default-features = false, features = ["alloc"] }
```

**Note**: The `nom` dependency always includes the `alloc` feature. When `std` feature is enabled, `nom/std` takes precedence, but `alloc` remains available. When `std` is disabled, `nom` uses only the `alloc` feature.

## Testing Strategy

1. **Compilation testing**:

   - Verify `no_std` builds compile successfully
   - Test feature combinations (std vs no_std)
   - Note: Runtime tests in `no_std` environment are not required for initial implementation

2. **CI/CD**:

   - Add `no_std` compilation checks to CI
   - Ensure both `std` and `no_std` feature combinations build

3. **Documentation**:
   - Document feature flags
   - Provide `no_std` usage examples
   - Document minimum Rust version (1.73)

## Breaking Changes

**Note**: This implementation prioritizes getting `no_std` working over backward compatibility. For migration strategies and backward-compatible approaches, see `migration.md`.

### Confirmed Breaking Changes

1. **`ParseError::info` type**:

   - **Change**: `std::string::String` → `alloc::string::String`
   - **Impact**: Type changes, but API remains the same
   - **Affected**: All code using `ParseError::info`

2. **Collection types in AST**:

   - **Change**: `std::vec::Vec<T>` → `alloc::vec::Vec<T>`
   - **Impact**: Type changes throughout AST nodes
   - **Affected**: All AST types using `Vec<T>`:
     - `NonEmpty<T>` wrapper
     - `Subroutine(Vec<TypeName>)`
     - `FunCall(FunIdentifier, Vec<Expr>)`
     - `parameters: Vec<FunctionParameterDeclaration>`
     - `statement_list: Vec<Statement>`
     - And many more...

3. **Error trait implementation**:

   - **Change**: `std::error::Error` → `alloc::error::Error` (when `std` disabled)
   - **Impact**: Error trait changes based on feature flags
   - **Affected**: Code that relies on `std::error::Error` trait specifically

4. **Minimum Rust version**:

   - **Change**: Requires Rust 1.73+ (for `alloc::error::Error`)
   - **Impact**: Older Rust versions no longer supported
   - **Affected**: Projects using Rust < 1.73

5. **Import paths**:
   - **Change**: `std::*` → `alloc::*` or `core::*`
   - **Impact**: Internal changes, but may affect code that imports internals
   - **Affected**: Code that imports internal types directly

## Dependencies to Add

### Required for `no_std`:

- None (use `alloc` crate from standard library)
- `nom` with `alloc` feature (already a dependency)

## Estimated Effort

- **Implementation**: 2-3 days

  - Update dependencies and imports
  - Replace all `std::*` types with `alloc::*` or `core::*` equivalents
  - Update error trait implementation
  - Add `#![no_std]` attribute and extern crate
  - Update all `std::*` imports to conditional imports
  - Compilation testing

- **Testing and Documentation**: 1 day
  - Add `no_std` compilation checks
  - Update CI/CD
  - Documentation updates
  - Usage examples

**Total**: ~3-4 days of development work

## Decisions Made

1. **nom features**: Use `nom` with `alloc` feature

   - `nom = { version = "7", default-features = false, features = ["alloc"] }`
   - When `std` feature is enabled, `nom/std` will be used instead

2. **Minimum Rust version**: 1.73

   - Required for `alloc::error::Error` trait
   - Set `rust-version = "1.73"` in Cargo.toml

3. **Transpiler module**: Verify it works in `no_std`

   - Transpiler uses `core::fmt::Write` which is available in `no_std`
   - Should verify all transpiler functionality works correctly

4. **Testing**: Compilation checks only for now
   - Focus on ensuring `no_std` builds compile successfully
   - Runtime testing in `no_std` environment can be added later if needed

## Related Documents

- **Migration Plan**: See `migration.md` for backward compatibility strategies and migration approaches

## References

- [Rust `no_std` book](https://docs.rust-embedded.org/book/intro/no-std.html)
- [nom `no_std` support](https://github.com/rust-bakery/nom#no-std-support)
- [Rust `alloc` crate](https://doc.rust-lang.org/alloc/)
- [Rust `alloc::error::Error` (1.73+)](https://doc.rust-lang.org/alloc/error/trait.Error.html)
