# Zed Extension for fuzzy_ruby_server

**Date:** 2026-03-04
**Status:** Brainstorm complete

## What We're Building

A Zed editor extension that integrates [fuzzy_ruby_server](https://github.com/pheen/fuzzy_ruby_server) — a Rust-based Ruby language server optimized for large codebases — into Zed's LSP infrastructure.

The extension is a thin connector — it locates and launches the fuzzy_ruby_server binary so Zed can use it as an LSP. The server itself provides: Go to Definition, Workspace Symbol Search, Diagnostics, References, Highlights, and Rename.

Target languages: Ruby, ERB, and related Ruby file types (Gemfile, Rakefile) — must match Zed's internal language names.

## Why This Approach

**Approach B: Thin WASM extension expecting a pre-installed binary.**

We chose this because:
1. **WASM sandbox constraints** — Zed extensions run in a sandboxed WASM environment. The `zed_extension_api` provides `download_file()` for pre-built binaries and `latest_github_release()` for GitHub releases, but doesn't expose arbitrary shell execution (`git clone`, `cargo build`).
2. **No upstream releases** — fuzzy_ruby_server has no GitHub releases with prebuilt binaries. Without downloadable assets, the standard Zed pattern (download binary from release) doesn't apply.
3. **Simplicity** — A thin extension that locates the binary on `$PATH` or a configured path is fast to build, reliable, and easy to maintain.
4. **User base** — Ruby developers using this tool likely have Rust installed already (or can install it via rustup).

### How it works

1. User installs fuzzy_ruby_server: `cargo install --git https://github.com/pheen/fuzzy_ruby_server`
2. Extension activates on Ruby files
3. `language_server_command()` returns the path to the `fuzzy` binary
4. Zed starts the LSP server via stdio
5. All LSP features work through Zed's built-in LSP client

### Alternatives considered

- **Build from source in extension** — Not feasible due to WASM sandbox restrictions
- **Fork with CI releases** — Too much maintenance overhead for now
- **Auto-install rustup** — Can't run shell commands from WASM sandbox

## Key Decisions

1. **Binary discovery:** Look for `fuzzy` on `$PATH` (binary name needs verification during implementation — depends on Cargo.toml `[[bin]]` target). Allow override via Zed settings for binary path.
2. **No Tree-sitter grammar:** Rely on Zed's built-in Ruby language support. Our extension only adds the LSP server.
3. **Version tracking:** Always use latest master branch. Users update by re-running `cargo install --git`.
4. **Language scope:** Register for Ruby, ERB, and related Ruby file types. Must use Zed's internal language names (verify during implementation).
5. **Communication:** stdio (the server reads stdin, writes stdout — standard LSP transport).
6. **No CLI arguments:** The server takes no arguments; configuration comes via LSP `InitializeParams`.

## Extension Structure

```
zed-fuzzy-ruby-server/
  extension.toml          # Extension manifest
  Cargo.toml              # Rust WASM build config
  src/lib.rs              # Extension implementation (language_server_command)
  LICENSE                 # Required for Zed extension registry
```

No `languages/` directory needed since we rely on Zed's built-in Ruby support.

## Open Questions

None — all questions resolved during brainstorm.
