# Migration Plan: Backward Compatibility for `no_std` Support

## Overview

This document outlines strategies for maintaining backward compatibility when adding `no_std` support to the GLSL parser. This is separate from the main `no_std` implementation plan, which prioritizes getting a working proof of concept.

## Goals

- Maintain API compatibility with existing `std`-based code
- Provide a smooth migration path for users
- Minimize breaking changes in the public API
- Allow gradual adoption of `no_std` features

## Backward Compatibility Strategies

### Strategy 1: Type Aliases (Recommended)

Use type aliases to maintain API compatibility while allowing `no_std` support.

#### Implementation

Create a compatibility module that provides type aliases:

```rust
// In glsl/src/compat.rs or glsl/src/lib.rs

#[cfg(feature = "std")]
pub use std::vec::Vec;
#[cfg(not(feature = "std"))]
pub use alloc::vec::Vec;

#[cfg(feature = "std")]
pub use std::string::String;
#[cfg(not(feature = "std"))]
pub use alloc::string::String;

#[cfg(feature = "std")]
pub use std::error::Error;
#[cfg(not(feature = "std"))]
pub use alloc::error::Error;
```

Then use these throughout the codebase:

```rust
use crate::compat::{Vec, String, Error};

pub struct ParseError {
    pub info: String,  // Compatible type alias
}

pub struct NonEmpty<T>(pub Vec<T>);  // Compatible type alias
```

#### Pros
- Zero API changes for users
- Transparent to existing code
- Easy to implement

#### Cons
- Type aliases don't hide the underlying type completely
- Some advanced use cases may still see the underlying type

### Strategy 2: Feature-Gated Type Exports

Export types conditionally based on features, maintaining the same public API.

#### Implementation

```rust
// In glsl/src/lib.rs

#[cfg(feature = "std")]
pub use std::string::String as ParseErrorString;
#[cfg(not(feature = "std"))]
pub use alloc::string::String as ParseErrorString;

pub struct ParseError {
    pub info: ParseErrorString,
}
```

#### Pros
- Explicit about what's being used
- Can provide different APIs for different features

#### Cons
- More verbose
- May require more documentation

### Strategy 3: Wrapper Types

Create wrapper types that abstract over `std` vs `alloc` implementations.

#### Implementation

```rust
pub struct CompatString {
    #[cfg(feature = "std")]
    inner: std::string::String,
    #[cfg(not(feature = "std"))]
    inner: alloc::string::String,
}

impl CompatString {
    pub fn new() -> Self { /* ... */ }
    pub fn as_str(&self) -> &str { /* ... */ }
    // ... other methods
}
```

#### Pros
- Complete abstraction
- Can provide unified API

#### Cons
- Significant refactoring required
- May have performance overhead
- More complex to maintain

## Recommended Approach

**Use Strategy 1 (Type Aliases)** - It provides the best balance of:
- Minimal code changes
- Maximum compatibility
- Easy to understand and maintain

## Migration Steps for Users

### Step 1: Update Dependencies

```toml
[dependencies]
glsl = { version = "7.0", default-features = false, features = ["std"] }
```

### Step 2: For `no_std` Users

```toml
[dependencies]
glsl = { version = "7.0", default-features = false }
# Ensure allocator is available in your no_std environment
```

### Step 3: Update Code (if needed)

Most code should work without changes. However, if you're doing type comparisons or pattern matching:

```rust
// Before (may break)
if let std::string::String = error.info { }

// After (works with both)
if let String = error.info { }  // Uses type alias
```

## Feature Flag Strategy

### Recommended Feature Flags

```toml
[features]
default = ["std"]
std = ["nom/std"]
# When std is disabled, alloc is used automatically
```

### User Migration

**For `std` users (no changes needed)**:
```toml
glsl = "7.0"  # Uses std by default
```

**For `no_std` users**:
```toml
glsl = { version = "7.0", default-features = false }
```

## Versioning Strategy

### Option 1: Major Version Bump

- Current: `7.0.0` (with `std`)
- Next: `8.0.0` (with `no_std` support, potentially breaking)

**Pros**: Clear breaking change signal
**Cons**: May be too aggressive if compatibility is maintained

### Option 2: Minor Version Bump

- Current: `7.0.0` (with `std`)
- Next: `7.1.0` (with `no_std` support, backward compatible)

**Pros**: Maintains compatibility, easier adoption
**Cons**: Less clear about potential issues

### Option 3: Feature Flag Only

- Keep same version, add `no_std` support behind feature flag
- `std` remains default, `no_std` is opt-in

**Pros**: Zero breaking changes
**Cons**: May not be possible if types change

## Testing Strategy for Compatibility

1. **API Compatibility Tests**:
   - Test that all public APIs work the same way
   - Test that types are compatible
   - Test that error handling works identically

2. **Integration Tests**:
   - Test with existing user code
   - Test feature flag combinations
   - Test both `std` and `no_std` modes

3. **Documentation**:
   - Document feature flags clearly
   - Provide migration examples
   - Document any differences between modes

## Potential Issues and Solutions

### Issue 1: Type Identity

**Problem**: `std::string::String` and `alloc::string::String` are different types, even if compatible.

**Solution**: Use type aliases so users don't need to know the difference.

### Issue 2: Error Trait Differences

**Problem**: `std::error::Error` vs `alloc::error::Error` may have different methods.

**Solution**: Conditionally implement both, or use a trait alias.

### Issue 3: Serialization

**Problem**: Serialization libraries may expect `std::string::String`.

**Solution**: Most serialization libraries work with both. Document any exceptions.

## Timeline

1. **Phase 1**: Implement `no_std` support (see `no_std.md`)
2. **Phase 2**: Add type aliases for compatibility (this document)
3. **Phase 3**: Test compatibility with existing code
4. **Phase 4**: Release with migration guide

## References

- [Rust Edition Guide - Compatibility](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
- [Semantic Versioning](https://semver.org/)
- [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)

