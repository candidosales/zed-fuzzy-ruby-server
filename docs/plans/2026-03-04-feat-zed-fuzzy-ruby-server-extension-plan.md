---
title: "feat: Zed extension for fuzzy_ruby_server"
type: feat
status: completed
date: 2026-03-04
deepened: 2026-03-04
origin: docs/brainstorms/2026-03-04-zed-fuzzy-ruby-server-brainstorm.md
---

# feat: Zed Extension for fuzzy_ruby_server

## Enhancement Summary

**Deepened on:** 2026-03-04
**Sections enhanced:** 7
**Research agents used:** architecture-strategist, code-simplicity-reviewer, security-sentinel, pattern-recognition-specialist, framework-docs-researcher, best-practices-researcher

### Key Improvements
1. **Simplified code** — removed YAGNI methods, flattened nested if-let chain, trimmed error message
2. **Fixed Cargo.toml naming** — `zed_fuzzy_ruby_server` (underscores) per ecosystem convention
3. **Security documented** — CVE-2025-68432 awareness, Worktree Trust requirement
4. **WASM target clarified** — `wasm32-wasip2` is the current target (not wasip1)
5. **Unified argument handling** — both discovery branches now read `binary.arguments` from settings
6. **Testing strategy detailed** — concrete debug commands and verification steps

### New Considerations Discovered
- Extension IDs cannot start with `zed-` (registry enforces this) — our `fuzzy-ruby-server` ID is correct
- `zed_extension_api` 0.7.0 is confirmed current; `schema_version = 1` is correct
- Binary name `fuzzy` is generic and collision-prone — README must include verification step
- Zed ≥ 0.218.2 required for Worktree Trust (security protection against malicious `.zed/settings.json`)

---

## Overview

Build a thin Zed WASM extension that locates a pre-installed `fuzzy_ruby_server` binary and registers it as an LSP for Ruby files. The extension is a connector only — it finds the binary, tells Zed to start it via stdio, and Zed handles all LSP communication.

(see brainstorm: docs/brainstorms/2026-03-04-zed-fuzzy-ruby-server-brainstorm.md)

## Acceptance Criteria

- [x] Extension loads in Zed and registers a language server for Ruby files
- [x] `language_server_command()` finds `fuzzy` binary on PATH via `worktree.which()`
- [x] Users can override binary path via Zed LSP settings
- [x] Clear error message when binary is not found, with install instructions
- [x] Extension works alongside Zed's built-in Ruby support (no Tree-sitter grammar bundled)
- [x] README documents: installation, settings, conflict resolution with ruby-lsp/solargraph, security
- [x] MIT LICENSE file included

## Implementation Plan

### Step 1: Verify binary name and behavior

Before writing any code, confirm the binary name produced by `cargo install --git https://github.com/pheen/fuzzy_ruby_server`:

```bash
cargo install --git https://github.com/pheen/fuzzy_ruby_server
which fuzzy  # expected: ~/.cargo/bin/fuzzy
```

Verify it starts via stdio with no arguments and accepts LSP `initialize`.

#### Research Insights

**Binary name collision risk:** The name `fuzzy` is generic. Other tools (fzf-related, npm packages) could share this name. The README must include a verification step: confirm the resolved path is `~/.cargo/bin/fuzzy` (or equivalent), not a different tool.

**Verification method:** Since fuzzy_ruby_server has no `--version` flag, verify by checking the binary path:
```bash
which fuzzy  # should point to ~/.cargo/bin/fuzzy
file $(which fuzzy)  # should show Mach-O or ELF binary
```

### Step 2: Create extension.toml

```toml
id = "fuzzy-ruby-server"
name = "Fuzzy Ruby Server"
description = "Integrates fuzzy_ruby_server — a fast Ruby language server for large codebases."
version = "0.1.0"
schema_version = 1
authors = ["Your Name <your@email.com>"]
repository = "https://github.com/your-username/zed-fuzzy-ruby-server"

[language_servers.fuzzy-ruby-server]
name = "Fuzzy Ruby Server"
languages = ["Ruby"]
```

**Decision: Start with `"Ruby"` only.** ERB support (`"ERB"`, `"HTML+ERB"`) requires verifying that fuzzy_ruby_server handles `languageId: "erb"` correctly. Add ERB languages in a follow-up after testing.

(see brainstorm: key decision #4 — language scope)

#### Research Insights

**Naming rules (enforced by registry CI):**
- Extension IDs: lowercase, alphanumeric, hyphens only — no underscores
- Cannot start with `zed-` or end with `-zed`
- `schema_version = 1` is the current and only version
- `languages = ["Ruby"]` (plural, array) is the correct format for API 0.7.0

**Verified language names:** Zed's internal Ruby language names are: `"Ruby"`, `"ERB"`, `"HTML+ERB"`, `"YAML+ERB"`, `"JS+ERB"`. The `"Ruby"` name covers `.rb`, `Gemfile`, `Rakefile`, `.gemspec` files automatically.

**`language_ids` mapping:** Not needed for Ruby-only registration. Only required when the LSP server expects different `languageId` values than Zed's internal names.

### Step 3: Create Cargo.toml

```toml
[package]
name = "zed_fuzzy_ruby_server"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
path = "src/lib.rs"
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.7.0"
```

#### Research Insights

**Package name convention:** Use underscores (`zed_fuzzy_ruby_server`), not hyphens. This matches the ecosystem convention (`zed_standardrb`, `zed_ruby`).

**`crate-type = ["cdylib"]`** is required for WASM compilation — without it, the binary cannot be loaded as a WASM component.

**API version confirmed:** `zed_extension_api = "0.7.0"` is the latest version (compatible with Zed ≥ 0.205.x).

**WASM target:** Extensions now compile to `wasm32-wasip2` (WASI Component Model, Rust Tier 2 since 1.82). Ensure rustup has the target: `rustup target add wasm32-wasip2`.

### Step 4: Implement src/lib.rs

```rust
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct FuzzyRubyExtension {}

impl zed::Extension for FuzzyRubyExtension {
    fn new() -> Self {
        Self {}
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree).ok();

        // Check user-configured binary path
        if let Some((path, args)) = settings
            .as_ref()
            .and_then(|s| s.binary.as_ref())
            .and_then(|b| {
                b.path
                    .as_ref()
                    .map(|p| (p.clone(), b.arguments.clone().unwrap_or_default()))
            })
        {
            return Ok(zed::Command {
                command: path,
                args,
                env: worktree.shell_env(),
            });
        }

        // Find on PATH
        let binary_name = "fuzzy";
        let path = worktree.which(binary_name).ok_or_else(|| {
            format!(
                "`{binary_name}` not found on PATH. \
                 Install with: cargo install --git https://github.com/pheen/fuzzy_ruby_server\n\
                 Or set a custom path in settings under \
                 \"lsp\" > \"fuzzy-ruby-server\" > \"binary\" > \"path\"."
            )
        })?;

        // Read args from settings even on PATH branch (unified handling)
        let args = settings
            .and_then(|s| s.binary)
            .and_then(|b| b.arguments)
            .unwrap_or_default();

        Ok(zed::Command {
            command: path,
            args,
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(FuzzyRubyExtension);
```

**Key design decisions in the code:**

1. **Settings schema:** Uses `LspSettings::for_worktree()` which reads from `"lsp": { "fuzzy-ruby-server": { "binary": { "path": "..." } } }`. This is the standard Zed pattern — no custom struct needed.
2. **Unified argument handling:** Both discovery branches read `binary.arguments` from settings. This means users can add CLI arguments without specifying a redundant explicit path.
3. **Shell env:** Always passes `worktree.shell_env()` so the server inherits correct PATH, GEM_HOME, etc. This is load-bearing — fuzzy_ruby_server needs these to resolve gem paths.
4. **Flat chain:** Uses `and_then` chain instead of nested `if let` — idiomatic Rust, less indentation.
5. **No `language_server_initialization_options` or `language_server_workspace_configuration`:** The server takes no special config. The trait's default `Ok(None)` is correct. Add them only if the server gains a config surface.
6. **Struct style:** `struct FuzzyRubyExtension {}` matches ecosystem convention (standardrb, official ruby).

#### Research Insights

**Simplicity review findings:**
- Removed `language_server_initialization_options` and `language_server_workspace_configuration` — YAGNI. Default `Ok(None)` is correct for a server with no config surface. (~14 lines removed)
- Trimmed error message — macOS PATH caveat and inline JSON belong in README, not runtime errors
- Flattened triple-nested `if let` to `and_then` chain

**Architecture review findings:**
- Two-tier binary discovery (settings → PATH) is the correct pattern for a Rust binary
- `worktree.shell_env()` is actively better than `Default::default()` for Ruby LSPs — the server needs Ruby environment vars
- Unified argument handling across both branches prevents future footgun if server adds CLI args

**Pattern compliance:**
- `struct FuzzyRubyExtension {}` (not unit struct) matches standardrb/official-ruby convention
- `register_extension!` macro correctly placed at module level
- `language_server_id.as_ref()` key matches `[language_servers.fuzzy-ruby-server]` in extension.toml
- `label_for_completion`/`label_for_symbol` correctly omitted — add only after testing server response schema

**Security review findings:**
- `zed::Command` uses separate `command`, `args`, `env` fields — no shell interpolation. This correctly prevents command injection even with malicious settings.
- `LspSettings::for_worktree()` reads from project-local `.zed/settings.json` — this was the attack vector for CVE-2025-68432. Mitigated by Zed ≥ 0.218.2 Worktree Trust mechanism. No code change needed in the extension.
- PATH-based discovery has theoretical poisoning risk (malicious `fuzzy` earlier on PATH). Low severity; settings override is the escape hatch.

### Step 5: Add LICENSE

Add MIT license file (same as fuzzy_ruby_server). This is required for Zed extension registry submission (enforced since October 2025).

### Step 6: Write README

The README must cover:

1. **Prerequisites:** `cargo install --git https://github.com/pheen/fuzzy_ruby_server`
2. **Verify installation:** `which fuzzy` should return `~/.cargo/bin/fuzzy` (verify exact path, not just existence, due to generic binary name)
3. **Custom binary path:** Settings JSON example for `lsp.fuzzy-ruby-server.binary.path`
4. **macOS Dock users:** Add `export PATH="$HOME/.cargo/bin:$PATH"` to `~/.zshenv` (not `.zshrc` — Dock launches don't source interactive shell config)
5. **Co-existence with ruby-lsp/solargraph:** How to disable other Ruby LSPs:
   ```json
   {
     "languages": {
       "Ruby": {
         "language_servers": ["fuzzy-ruby-server", "!ruby-lsp", "!solargraph", "..."]
       }
     }
   }
   ```
6. **Updating:** Re-run `cargo install --git ...` then restart the language server via command palette (`editor: restart language server`)
7. **Capabilities:** List what fuzzy_ruby_server provides (definition, references, diagnostics, highlights, rename, workspace symbols)
8. **Known limitations:** Initial indexing takes time on large codebases (600s reindex cycle); no completions or hover docs
9. **Security:** Only open projects with `.zed/settings.json` files you trust. Zed ≥ 0.218.2 enforces a trust prompt.

#### Research Insights

**README best practice (from registry conventions):** Include feature list, requirements section, configuration with `settings.json` examples, known limitations, and link to upstream project.

**The `"..."` wildcard:** Explain that `"..."` includes all other registered servers at that position. Without it, only the explicitly listed servers will run.

### Step 7: Test as dev extension

1. **Install dev extension:** Open Zed, go to Extensions > Install Dev Extension, point to the project directory
   - **Prerequisite:** Rust must be installed via **rustup** (not Homebrew). Dev extension compilation silently fails otherwise.
   - Ensure WASM target is installed: `rustup target add wasm32-wasip2`

2. **Verify LSP activation:** Open a Ruby project and check:
   - Go to Definition works (F12 / Cmd+Click)
   - Workspace symbols work (Cmd+T)
   - Diagnostics appear in editor
   - Status bar shows "Fuzzy Ruby Server" as active language server

3. **Test failure case:** Rename `fuzzy` binary temporarily → open a `.rb` file → verify error message appears with install instructions

4. **Test settings override:** Add custom path to `settings.json`:
   ```json
   { "lsp": { "fuzzy-ruby-server": { "binary": { "path": "/path/to/fuzzy" } } } }
   ```
   Verify it takes precedence over PATH lookup.

5. **Test macOS Dock launch:** Quit Zed completely, launch from Dock (not terminal), open a Ruby project, verify server activates. If it fails, verify the `~/.zshenv` PATH workaround resolves it.

6. **Test co-existence:** Install with ruby-lsp also active. Verify both servers run. Test disabling ruby-lsp via `language_servers` config.

#### Research Insights

**Debugging commands:**

| Command | Purpose |
|---|---|
| `zed --foreground` | Run Zed with verbose logging to terminal stdout |
| `debug: open language server logs` | View stdout/stderr from LSP processes |
| `zed: open log` | Full Zed application log |
| `editor: restart language server` | Restart LSP without full editor restart |

**Known testing gotchas:**
- `println!` and `dbg!` in WASM code forward to Zed stdout — only visible with `zed --foreground`
- If "Install Dev Extension" appears to do nothing, verify Rust is from rustup
- If macOS Finder has the extensions folder open, extension loading can fail (issue #8096)

## File Structure

```
zed-fuzzy-ruby-server/
  extension.toml          # Extension manifest
  Cargo.toml              # WASM build config (cdylib, zed_extension_api 0.7.0)
  src/lib.rs              # Extension implementation (~40 lines)
  LICENSE                 # MIT license (required for registry)
  README.md               # User documentation
```

## Decisions Carried from Brainstorm

1. **Thin WASM extension** — no build-from-source, no binary downloading (brainstorm: approach B)
2. **Binary on PATH** with settings override (brainstorm: key decision #1)
3. **No Tree-sitter grammar** — rely on Zed built-in Ruby (brainstorm: key decision #2)
4. **stdio transport, no CLI args** (brainstorm: key decisions #5, #6)
5. **Start with Ruby only** — ERB support deferred until verified (refined from brainstorm key decision #4 based on SpecFlow analysis)

## Resolved from SpecFlow Analysis

| Gap | Resolution |
|-----|-----------|
| Binary-not-found error is terse | Concise error message with install command and settings hint (macOS detail in README) |
| macOS Dock PATH mismatch | Documented in README with `~/.zshenv` workaround; settings override as escape hatch |
| ERB language registration ambiguity | Start with "Ruby" only; add ERB after verifying server handles `languageId: "erb"` |
| Settings schema undefined | Use standard `LspSettings` pattern — no custom schema needed |
| Conflict with ruby-lsp/solargraph | Document `language_servers` config in README with `"..."` wildcard explanation |
| Update flow undocumented | README covers `cargo install` + `editor: restart language server` |

## Resolved from Deepening Research

| Finding | Source | Resolution |
|---------|--------|-----------|
| Cargo.toml name should use underscores | pattern-recognition | Changed to `zed_fuzzy_ruby_server` |
| `initialization_options`/`workspace_configuration` are YAGNI | code-simplicity | Removed — trait defaults return `Ok(None)` |
| Error message too verbose for runtime | code-simplicity | Trimmed; macOS details moved to README |
| Arguments not unified across branches | architecture-strategist | PATH branch now also reads `binary.arguments` |
| CVE-2025-68432 binary path override | security-sentinel | Documented in README; no code change needed (Zed platform fix) |
| `struct` should use `{}` not unit style | pattern-recognition | Changed to `struct FuzzyRubyExtension {}` |
| WASM target is now `wasm32-wasip2` | best-practices | Documented in Step 7 prerequisites |
| Generic binary name `fuzzy` collision risk | architecture-strategist + security-sentinel | README verification step checks exact path |

## Publishing Checklist (Future)

When ready to publish to the Zed extension registry:

- [ ] Verify `id` in extension.toml is unique in the registry
- [ ] Ensure LICENSE file is present (MIT)
- [ ] Fork `zed-industries/extensions` repo
- [ ] Add as git submodule with HTTPS URL (not SSH)
- [ ] Add entry to `extensions.toml` (alphabetically sorted)
- [ ] Run `pnpm sort-extensions`
- [ ] Open PR — CI validates automatically

## Sources

- **Origin brainstorm:** [docs/brainstorms/2026-03-04-zed-fuzzy-ruby-server-brainstorm.md](docs/brainstorms/2026-03-04-zed-fuzzy-ruby-server-brainstorm.md)
- **Zed extension API:** [docs.rs/zed_extension_api/0.7.0](https://docs.rs/zed_extension_api/0.7.0)
- **Reference extension (standardrb):** [github.com/himynameisjonas/zed-standardrb](https://github.com/himynameisjonas/zed-standardrb)
- **Reference extension (official Ruby):** [github.com/zed-extensions/ruby](https://github.com/zed-extensions/ruby)
- **fuzzy_ruby_server:** [github.com/pheen/fuzzy_ruby_server](https://github.com/pheen/fuzzy_ruby_server)
- **macOS PATH issue:** [github.com/zed-industries/zed/issues/44291](https://github.com/zed-industries/zed/issues/44291)
- **CVE-2025-68432:** [github.com/zed-industries/zed/security/advisories/GHSA-29cp-2hmh-hcxj](https://github.com/zed-industries/zed/security/advisories/GHSA-29cp-2hmh-hcxj)
- **Zed Worktree Trust:** [zed.dev/blog/secure-by-default](https://zed.dev/blog/secure-by-default)
- **Zed extension registry:** [github.com/zed-industries/extensions](https://github.com/zed-industries/extensions)
- **Life of a Zed Extension (blog):** [zed.dev/blog/zed-decoded-extensions](https://zed.dev/blog/zed-decoded-extensions)
