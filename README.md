# bend-language-server

Language server for the Bend programming language based on the Language Server Protocol (LSP).

## Features

The current features implemented in this project are

- Semantic token highlighting
  - Code highlighting using the [Bend tree sitter grammar](https://github.com/higherOrderCO/tree-sitter-bend)
- Diagnostic reporting
  - Reports compilation warnings, errors, and other information

We accept contributions and feature requests!

## Development

Currently, the language server is only developed and tested for VSCode. Feel free to add contributions specific to other code editors!

### VSCode extension

This project requires the [Rust toolchain](https://rustup.rs) and [pnpm](https://pnpm.io) to build.

1. Prepare the Javascript environment with `pnpm i`.
2. Build the language server with `cargo build`.
3. Open the project with VSCode. Press <kbd>F5</kbd> or click <kbd>Launch Client</kbd> in the Debug panel.

This should result in a new VSCode window being opened with the `bend-language-server` extension loaded.
