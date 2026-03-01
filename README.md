# OpenGL Shading Language parser and AST manipulation crates

> **Note:** This is a [LightPlayer](https://github.com/light-player/lightplayer) fork of the original [glsl-parser](https://git.sr.ht/~hadronized/glsl) project.

This repository holds the [glsl] and [glsl-quasiquote] projects. Feel free to visit each projects
for more information.

## Purpose of fork

This fork adds the following features to support LightPlayer's GLSL compiler needs:

- **Span Support**: Source location tracking (`SourceSpan`) for improved error reporting and diagnostics
- **`no_std` Support**: Compatibility with embedded and RISC-V environments (e.g., ESP32 microcontrollers)

These enhancements enable LightPlayer to compile GLSL shaders to native RISC-V code on embedded devices while providing accurate source location information for error messages.

## Links

- **[LightPlayer](https://github.com/light-player/lightplayer)** - Main LightPlayer repository
- **[Original Upstream](https://git.sr.ht/~hadronized/glsl)** - Original glsl-parser project

[glsl]: ./glsl/README.md
[glsl-quasiquote]: ./glsl-quasiquote/README.md
