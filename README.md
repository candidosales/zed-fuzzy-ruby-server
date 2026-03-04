# Fuzzy Ruby Server for Zed

A [Zed](https://zed.dev) extension that integrates [fuzzy_ruby_server](https://github.com/pheen/fuzzy_ruby_server) — a fast Ruby language server designed for large codebases.

## Features

Provided by fuzzy_ruby_server:

- **Go to Definition** — Jump to method, variable, class, or module definitions
- **Workspace Symbol Search** — Search definitions across the project (Cmd+T)
- **Diagnostics** — Static analysis issues surfaced in the editor
- **References** — Find all usages of a symbol in the current file
- **Highlights** — Visual highlighting of symbol occurrences
- **Rename** — Batch rename symbols within a file

## Prerequisites

Install `fuzzy_ruby_server` via Cargo:

```bash
cargo install --git https://github.com/doompling/fuzzy_ruby_server
```

Verify the installation:

```bash
which fuzzy
# Expected: ~/.cargo/bin/fuzzy
```

> **Note:** The binary is named `fuzzy`. Since this is a generic name, confirm the path points to `~/.cargo/bin/fuzzy` (or your Cargo bin directory) and not a different tool.

### Rust Toolchain

If you don't have Rust installed, get it via [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Configuration

### Custom Binary Path

If `fuzzy` is not on your PATH, set a custom path in Zed settings (`settings.json`):

```json
{
  "lsp": {
    "fuzzy-ruby-server": {
      "binary": {
        "path": "/path/to/fuzzy"
      }
    }
  }
}
```

### macOS Dock Users

When Zed is launched from the Dock (not the terminal), `~/.cargo/bin` may not be in your PATH. Add it to `~/.zshenv`:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshenv
```

> **Why `.zshenv`?** Dock launches don't source `.zshrc` (interactive shell config). `.zshenv` is sourced for all shell invocations.

Alternatively, use the custom binary path setting above.

### Co-existence with Other Ruby LSPs

If you have `ruby-lsp` or `solargraph` installed, Zed runs all registered Ruby language servers simultaneously. To use only Fuzzy Ruby Server:

```json
{
  "languages": {
    "Ruby": {
      "language_servers": ["fuzzy-ruby-server", "!ruby-lsp", "!solargraph", "..."]
    }
  }
}
```

The `"..."` wildcard includes all other unspecified servers. Use `!server-name` to exclude specific servers.

## Updating

Re-run the install command to get the latest version:

```bash
cargo install --git https://github.com/pheen/fuzzy_ruby_server
```

Then restart the language server in Zed via the command palette: **Editor: Restart Language Server**.

## Known Limitations

- **Initial indexing:** The server indexes your workspace on startup and reindexes every 10 minutes. On large codebases, features like workspace symbol search may be incomplete until the first index finishes.
- **No completions or hover:** fuzzy_ruby_server focuses on navigation and diagnostics. For completion and hover documentation, consider running it alongside `ruby-lsp`.
- **File-scoped references and rename:** References and rename operate within the current file only.

## Security

This extension locates and launches a binary from your system. Only open projects with `.zed/settings.json` files you trust — project-level settings can override the binary path. Zed >= 0.218.2 enforces a trust prompt for project settings.

## License

MIT
