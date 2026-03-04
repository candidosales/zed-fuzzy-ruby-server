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

        let binary_name = "fuzzy";
        let path = worktree.which(binary_name).ok_or_else(|| {
            format!(
                "`{binary_name}` not found on PATH. \
                 Install with: cargo install --git https://github.com/pheen/fuzzy_ruby_server\n\
                 Or set a custom path in settings under \
                 \"lsp\" > \"fuzzy-ruby-server\" > \"binary\" > \"path\"."
            )
        })?;

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
