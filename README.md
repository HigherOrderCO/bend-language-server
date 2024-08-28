# bend-language-server

Language server for the Bend programming language based on the Language Server Protocol (LSP).

## Features

The current features implemented in this language server are

- Semantic token highlighting
  - Code highlighting using the [Bend tree sitter grammar](https://github.com/higherOrderCO/tree-sitter-bend)
- Diagnostic reporting
  - Reports compilation warnings, errors, and other information

We accept contributions and feature requests!

## Installation

### VSCode extension

As of this alpha version, we still don't publish compiled releases of the language server, so you'll need to install the [Rust toolchain](https://rustup.rs) to compile it. Afterwards, install the VSCode extension and open a `.bend` file.

On startup, the extension will ask you if you want it to automatically install the language server executable or if you want to set it up manually with the PATH environment variable. If you choose automatically, the extension will use `cargo` to install it to its local storage. If you want to install it manually, run the following command:

```
cargo install bend-language-server --version 0.2.37-alpha.2
```

Managing the language server manually will require you to update the language server yourself as new versions are published.

### Other editors

We are still not officially supporting other editors; however, if your editor of choice has support for the Language Server Protocol (LSP), you can try plugging it and our language server together. To install the LSP-compliant language server binary, use the Cargo command from the [Rust toolchain](https://rustup.rs):

```
cargo install bend-language-server --version 0.2.37-alpha.2
```

If the toolchain is correctly installed, `bend-language-server` should now be in your path.

We also have a [tree-sitter grammar](https://github.com/HigherOrderCO/tree-sitter-bend) for syntax highlighting with configuration instructions for Neovim.

## Development

Currently, the language server is only developed and tested for VSCode. Feel free to add contributions specific to other code editors!

### VSCode extension

This project requires the [Rust toolchain](https://rustup.rs) and [pnpm](https://pnpm.io) to build.

1. Prepare the Javascript environment with `pnpm i`.
2. Build the language server with `cargo build`.
3. Open the project with VSCode. Press <kbd>F5</kbd> or click <kbd>Launch Client</kbd> in the Debug panel.

This should result in a new VSCode window being opened with the `bend-language-server` extension loaded.
