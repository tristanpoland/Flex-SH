# Flex-SH Configuration Guide

Flex-SH is highly customizable through TOML configuration files. This guide explains all available configuration options.

## Configuration File Locations

Flex-SH looks for configuration files in the following order:

1. Path specified with `--config` flag
2. `$XDG_CONFIG_HOME/flex-sh/config.toml` (Linux/macOS)
3. `~/.config/flex-sh/config.toml` (Linux/macOS)
4. `%APPDATA%\flex-sh\config.toml` (Windows)
5. `config.toml` in the current directory

## Quick Start

Copy `example-config.toml` to your config directory:

```bash
# Linux/macOS
mkdir -p ~/.config/flex-sh
cp example-config.toml ~/.config/flex-sh/config.toml

# Windows
mkdir %APPDATA%\flex-sh
copy example-config.toml %APPDATA%\flex-sh\config.toml
```

## Configuration Sections

### 🎨 Shell Appearance (`[shell]`)

Control the look and feel of your shell:

- **`colors_enabled`**: Enable/disable colored output globally
- **`prompt_format`**: Define your prompt with variables like `{user}`, `{hostname}`, `{cwd}`
- **`colored_prompt`**: Raw ANSI escape sequence prompt (overrides `prompt_format`)
- **`show_git_branch`**: Display git branch in prompt when in a repository
- **`welcome_message`**: Custom message displayed on startup

**Example Prompts:**
```toml
# Simple prompt
prompt_format = "[{user}@{hostname} {cwd}]$ "

# Colored prompt with git integration
colored_prompt = "\\[\033[1;32m\\]{user}\\[\033[0m\\]@\\[\033[1;34m\\]{hostname}\\[\033[0m\\]:\\[\033[1;33m\\]{cwd}\\[\033[0m\\]$ "
```

### 📚 History Management (`[history]`)

Configure command history behavior:

- **`max_entries`**: Maximum commands to remember (default: 10000)
- **`ignore_duplicates`**: Skip consecutive duplicate commands
- **`ignore_space_prefix`**: Don't save commands starting with space
- **`ignore_patterns`**: Regex patterns for commands to never save
- **`fuzzy_search`**: Enable fuzzy history searching

### 🔤 Tab Completion (`[completion]`)

Customize auto-completion behavior:

- **`case_sensitive`**: Whether completion is case-sensitive
- **`show_hidden`**: Include hidden files in completion
- **`max_candidates`**: Maximum completions to display
- **`custom`**: Define completions for specific commands

**Custom Completions Example:**
```toml
[completion.custom]
git = ["add", "commit", "push", "pull", "status", "checkout"]
docker = ["run", "build", "ps", "images", "exec", "logs"]
```

### 🛠️ Built-in Commands (`[builtins]`)

Configure or disable built-in commands:

```toml
[builtins]
ls_enabled = true
cd_enabled = true

[builtins.ls]
default_args = ["-la"]
colors = true
human_readable = true

[builtins.cd]
expand_tilde = true
auto_create = false  # Ask before creating directories
```

### 🔗 Aliases (`[aliases]`)

Define command shortcuts:

```toml
[aliases]
ll = "ls -la"
gs = "git status"
".." = "cd .."
cls = "clear"
```

### 🎨 Color Schemes (`[colors]`)

Customize colors throughout the shell:

```toml
[colors]
scheme = "monokai"  # Built-in schemes: default, dark, light, monokai

[colors.custom]
success = "#00FF00"
error = "#FF0000"
directory = "#1E90FF"
executable = "#00FF00"
```

### ⌨️ Key Bindings (`[keybindings]`)

Customize keyboard shortcuts:

```toml
[keybindings]
edit_mode = "emacs"  # or "vi"

[keybindings.custom]
"Ctrl+R" = "reverse_search_history"
"Ctrl+L" = "clear_screen"
"Alt+." = "yank_last_arg"
```

**Available Actions:**
- `beginning_of_line`, `end_of_line`
- `forward_word`, `backward_word`
- `kill_line`, `unix_line_discard`
- `reverse_search_history`, `forward_search_history`
- `complete`, `menu_complete`
- `clear_screen`, `abort`

### 🔧 Performance (`[performance]`)

Optimize shell performance:

- **`cache_commands`**: Cache command lookups for faster execution
- **`max_background_processes`**: Limit concurrent background jobs
- **`auto_cleanup_zombies`**: Automatically clean up finished processes
- **`preload_commands`**: Commands to cache at startup

### 📝 Logging (`[logging]`)

Configure logging behavior:

```toml
[logging]
level = "info"  # error, warn, info, debug, trace
log_file = "~/.local/share/flex-sh/flex-sh.log"
file_logging = true
max_file_size = 10  # MB
```

### 🔒 Security (`[security]`)

Security-related settings:

- **`restrict_dangerous_commands`**: Ask for confirmation on dangerous operations
- **`sensitive_patterns`**: Patterns to exclude from history (passwords, tokens)
- **`max_command_length`**: Prevent buffer overflow attacks

```toml
[security]
dangerous_commands = ["rm -rf", "format", "dd"]
sensitive_patterns = ["password=", "token=", "api_key="]
```

### 🔌 Plugins (`[plugins]`)

Extension system configuration:

```toml
[plugins]
enabled = ["git-integration", "syntax-highlighting", "auto-suggestions"]

[plugins.git-integration]
show_status = true
show_branch = true

[plugins.syntax-highlighting]
highlight_commands = true
highlight_paths = true
```

### 🌍 Environment (`[environment]`)

Environment variable management:

```toml
[environment]
EDITOR = "nano"
PAGER = "less"

path_additions = ["~/.local/bin", "~/.cargo/bin"]
```

## Color Scheme Reference

### Built-in Schemes
- **`default`**: Standard terminal colors
- **`dark`**: Dark theme optimized for dark terminals
- **`light`**: Light theme for bright backgrounds
- **`monokai`**: Popular dark theme with vibrant colors
- **`solarized`**: Eye-friendly balanced color palette

### Custom Color Values
Colors can be specified as:
- Hex: `"#FF0000"`, `"#f00"`
- RGB: `"rgb(255, 0, 0)"`
- Named: `"red"`, `"blue"`, `"green"`
- ANSI codes: `"91"` (bright red)

## Advanced Configuration

### Conditional Configuration

Use different settings based on environment:

```toml
[shell]
# Default prompt
prompt_format = "{user}@{hostname}:{cwd}$ "

# Override for specific hostname
[shell.hostname."work-laptop"]
prompt_format = "[WORK] {user}:{cwd}$ "
```

### Per-Directory Settings

Configure behavior for specific directories:

```toml
[directory."/home/user/projects"]
[directory."/home/user/projects".aliases]
test = "cargo test"
build = "cargo build"
run = "cargo run"
```

### Plugin Development

Create custom plugins:

```toml
[plugins.my-plugin]
script = "~/.config/flex-sh/plugins/my-plugin.lua"
enabled = true
config = { key = "value" }
```

## Configuration Validation

Flex-SH validates configuration on startup and reports errors:

```bash
# Check configuration syntax
flex-sh --check-config

# Show current configuration
flex-sh --show-config
```

## Migration from Other Shells

### From Bash
```bash
# Convert .bashrc aliases
flex-sh --convert-bashrc ~/.bashrc >> ~/.config/flex-sh/config.toml
```

### From Zsh
```bash
# Convert .zshrc settings
flex-sh --convert-zshrc ~/.zshrc >> ~/.config/flex-sh/config.toml
```

### From Fish
```bash
# Convert fish configuration
flex-sh --convert-fish ~/.config/fish/config.fish >> ~/.config/flex-sh/config.toml
```

## Troubleshooting

### Common Issues

1. **Colors not working**: Check `colors.enabled = true` and terminal support
2. **History not saving**: Verify file permissions on history file
3. **Completions slow**: Reduce `completion.max_candidates` or disable `show_hidden`
4. **Plugins not loading**: Check plugin directory exists and permissions

### Debug Mode

Run with debug logging to troubleshoot:

```bash
FLEX_SH_LOG=debug flex-sh
```

### Performance Profiling

Profile shell startup:

```bash
flex-sh --profile-startup
```

## Environment Variables

Override config with environment variables:

```bash
export FLEX_SH_COLORS_ENABLED=false
export FLEX_SH_PROMPT_FORMAT="{cwd}$ "
export FLEX_SH_HISTORY_MAX_ENTRIES=5000
```

## Best Practices

1. **Start Small**: Begin with basic settings, gradually add customizations
2. **Version Control**: Keep your config in git for easy backup/sync
3. **Test Changes**: Use `--config` flag to test new configurations
4. **Performance**: Monitor startup time, disable unused features
5. **Security**: Be cautious with custom commands and plugins
6. **Backup**: Keep backups of working configurations

## Examples

### Minimal Configuration
```toml
[shell]
colors_enabled = true
prompt_format = "{user}:{cwd}$ "

[aliases]
ll = "ls -la"
".." = "cd .."
```

### Power User Configuration
```toml
[shell]
colors_enabled = true
colored_prompt = "\\[\033[1;36m\\][\\[\033[1;32m\\]{user}\\[\033[0m\\]@\\[\033[1;34m\\]{hostname}\\[\033[1;36m\\] \\[\033[1;33m\\]{cwd}\\[\033[1;36m\\]]\\[\033[1;35m\\]$\\[\033[0m\\] "
show_git_branch = true

[history]
max_entries = 50000
fuzzy_search = true
ignore_duplicates = true

[completion]
case_sensitive = false
show_hidden = true

[colors]
scheme = "monokai"

[plugins]
enabled = ["git-integration", "syntax-highlighting", "auto-suggestions"]

[keybindings]
edit_mode = "emacs"

[security]
restrict_dangerous_commands = true
```

For more examples and advanced configurations, see the `examples/` directory in the Flex-SH repository.