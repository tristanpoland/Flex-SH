# Flex-SH

A high-performance, modern system shell with rich features and cross-platform support.

## Features

### 🚀 Performance
- Written in Rust for maximum performance and memory safety
- Async/await architecture for non-blocking operations
- Efficient command parsing and execution

### 🎨 Rich Interface
- Syntax highlighting and colored output
- Multiple color schemes (default, dark, light, monokai)
- Cross-platform terminal support
- Advanced prompt customization

### 🔧 Built-in Commands
- **cd** - Change directory with OLDPWD support
- **echo** - Display text with variable expansion
- **ls** - List files with color coding and long format
- **pwd** - Print working directory
- **clear** - Clear terminal screen
- **env** - Environment variable management
- **which** - Locate commands in PATH
- **alias** - Command aliases (planned)
- **history** - Command history (planned)
- **help** - Built-in help system
- **exit** - Exit the shell

### 💡 Smart Features
- Command history with persistent storage
- Tab completion for commands and files
- Glob pattern expansion
- Environment variable expansion
- Input/output redirection
- Pipeline support
- Background processes
- Configuration system

## Installation

### From Source

```bash
git clone https://github.com/username/flex-sh.git
cd flex-sh
cargo build --release
```

The binary will be available at `target/release/flex-sh`.

### Using Cargo

```bash
cargo install flex-sh
```

## Usage

### Basic Usage

Start the shell:
```bash
flex-sh
```

Execute a single command:
```bash
flex-sh -c "ls -la"
```

Run a script:
```bash
flex-sh script.sh
```

### Command Line Options

```
flex-sh [OPTIONS] [SCRIPT]

Options:
  -c, --command <COMMAND>    Execute a single command and exit
  -i, --interactive          Interactive mode (default)
  -v, --verbose              Verbose output
      --no-color             Disable colors
      --config <CONFIG>      Configuration file path
  -h, --help                 Print help information
  -V, --version              Print version information

Arguments:
  <SCRIPT>  Script file to execute
```

## Configuration

Flex-SH looks for configuration files in the following order:
1. File specified by `--config` option
2. `~/.config/flex-sh/config.toml` (Unix/Linux/macOS)
3. `%APPDATA%\\flex-sh\\config.toml` (Windows)
4. `.flexsh.toml` in the current directory

### Example Configuration

```toml
[prompt]
format = "[{user}@{host} {cwd}]$ "
show_git = true
show_time = false
show_exit_code = true

[colors]
enabled = true
scheme = "monokai"
command_color = "bright_blue"
argument_color = "white"
error_color = "bright_red"
success_color = "bright_green"

[history]
max_entries = 10000
ignore_duplicates = true
ignore_space_prefixed = true

[completion]
enabled = true
case_sensitive = false
fuzzy_matching = true

[aliases]
ll = "ls -la"
la = "ls -A"
l = "ls -CF"

[environment]
EDITOR = "vim"
PAGER = "less"
```

## Built-in Commands

### Directory Navigation
```bash
cd                    # Go to home directory
cd /path/to/dir      # Change to specified directory
cd -                 # Go to previous directory
pwd                  # Print current directory
```

### File Operations
```bash
ls                   # List files
ls -la               # Long format with hidden files
ls -h                # Human-readable sizes
```

### System Information
```bash
env                  # Show all environment variables
env VAR=value        # Set environment variable
which command        # Find command location
```

### Shell Operations
```bash
help                 # Show all built-in commands
help command         # Show help for specific command
history              # Show command history (planned)
alias name=command   # Create alias (planned)
clear                # Clear screen
exit [code]          # Exit shell
```

## Advanced Features

### Redirection
```bash
command > file.txt       # Redirect output
command >> file.txt      # Append output
command < input.txt      # Redirect input
```

### Pipelines
```bash
ls | grep pattern        # Pipe output to grep
cat file | sort | uniq   # Multiple pipes
```

### Background Processes
```bash
sleep 10 &              # Run in background
```

### Variable Expansion
```bash
echo $HOME              # Environment variables
echo ${USER}            # Braced variables
```

### Glob Patterns
```bash
ls *.txt               # All .txt files
ls file?.log           # Single character wildcard
ls file[0-9].txt       # Character class
```

## Color Schemes

Flex-SH supports multiple color schemes:

- **default**: Bright colors optimized for dark terminals
- **dark**: Muted colors for dark terminals
- **light**: Colors optimized for light terminals
- **monokai**: Popular Monokai color scheme

Change color scheme in config:
```toml
[colors]
scheme = "monokai"
```

## Development

### Building

```bash
cargo build           # Debug build
cargo build --release # Release build
```

### Testing

```bash
cargo test            # Run all tests
cargo test --lib      # Library tests only
```

### Linting

```bash
cargo clippy          # Run linter
cargo fmt             # Format code
```

## Architecture

Flex-SH is built with a modular architecture:

- **Core**: Shell engine, parser, executor
- **Terminal**: Cross-platform terminal interface
- **Builtins**: Built-in command implementations
- **Utils**: Utilities for completion, path handling, etc.
- **Config**: Configuration management

### Module Structure

```
src/
├── main.rs              # Entry point
├── cli.rs               # Command line interface
├── core/                # Core shell functionality
│   ├── shell.rs         # Main shell loop
│   ├── parser.rs        # Command parsing
│   ├── executor.rs      # Command execution
│   └── history.rs       # Command history
├── terminal/            # Terminal interface
│   ├── mod.rs           # Terminal abstraction
│   ├── colors.rs        # Color scheme management
│   ├── events.rs        # Event handling
│   └── interface.rs     # Terminal UI
├── builtins/            # Built-in commands
│   ├── mod.rs           # Builtin command registry
│   ├── cd.rs            # Change directory
│   ├── ls.rs            # List files
│   └── ...              # Other builtins
├── config/              # Configuration system
│   ├── mod.rs           # Configuration types
│   └── settings.rs      # Settings management
└── utils/               # Utility modules
    ├── completion.rs    # Tab completion
    ├── path.rs          # Path utilities
    └── glob_expand.rs   # Glob expansion
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. Install Rust and Cargo
2. Clone the repository
3. Run tests: `cargo test`
4. Make your changes
5. Ensure tests pass
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] Enhanced tab completion with fuzzy matching
- [ ] Plugin system for extensibility
- [ ] Scripting language integration
- [ ] Remote shell capabilities
- [ ] Session management
- [ ] More built-in utilities
- [ ] Performance optimizations
- [ ] Windows-specific features
- [ ] Integration with system package managers