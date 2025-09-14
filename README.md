# 🚀 Flex-SH - A Modern, Feature-Rich System Shell

[![Rust](https://img.shields.io/badge/built_with-Rust-dea584.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Cross Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-green.svg)]()

**Flex-SH** is a modern, highly customizable system shell built in Rust that combines the best features of traditional shells with contemporary usability improvements. Designed for developers, system administrators, and power users who demand both performance and aesthetics.

---

## ✨ Features

### 🎨 **Beautiful & Customizable Interface**
- **Rich Unicode Prompts** - Gorgeous, informative prompts with full emoji support
- **Multiple Color Schemes** - Dark, light, monokai, and custom color themes
- **Smart Prompt Variables** - Dynamic user, hostname, directory, time, and git status
- **Responsive Design** - Adapts to terminal size and capabilities

### ⚡ **High Performance**
- **Blazing Fast** - Written in Rust for maximum performance and memory safety
- **Smart Tab Completion** - Intelligent completion for commands, paths, and arguments
- **Command Caching** - Frequently used commands are cached for instant execution
- **Async Architecture** - Non-blocking command execution with pipeline support

### 🔧 **Developer-Friendly**
- **Advanced History** - Fuzzy search, deduplication, and smart filtering
- **Git Integration** - Branch status, repository awareness, and shortcuts
- **Extensive Aliases** - Pre-configured shortcuts for git, development, and system commands
- **Pipeline Support** - Full support for command chaining and redirection

### 🌐 **Cross-Platform**
- **Universal Compatibility** - Native support for Windows, Linux, and macOS
- **PATH Resolution** - Intelligent executable discovery across all platforms
- **Windows Batch Support** - Seamless .bat, .cmd, and .exe execution on Windows
- **Unix Permissions** - Proper executable bit detection on Unix systems

---

## 🏃 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/Flex-SH.git
cd Flex-SH

# Build the release version
cargo build --release

# Run the shell
./target/release/flex-sh
```

### First Run

1. **Start the shell** - Run `flex-sh` to start your new shell experience
2. **Tab to complete** - Try typing `git st` and press Tab to see intelligent completion
3. **Explore commands** - Type `help` to see available built-in commands
4. **Customize** - Copy `flex-sh-config.toml` to set up your perfect environment

---

## 🎨 Beautiful Prompts

Flex-SH comes with stunning, informative prompts out of the box:

```bash
┌─[user@hostname]─[~/projects/flex-sh]
└─❯ git status

[user@hostname Flex-SH]$ ls -la

user@hostname:~/Documents$ npm run dev
```

### Prompt Variables

Customize your prompt with these dynamic variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `{user}` | Current username | `john` |
| `{hostname}` | System hostname | `dev-machine` |
| `{cwd}` | Current directory (home-relative) | `~/projects/app` |
| `{cwd_name}` | Just directory name | `app` |
| `{time}` | Current time | `14:30:25` |
| `{date}` | Current date | `2024-09-14` |
| `{git_branch}` | Git branch (when in repo) | `main` |

---

## ⚙️ Configuration

Flex-SH is highly configurable through TOML files. The shell looks for config files in this order:

1. Path specified with `--config` flag
2. `~/.config/flex-sh/config.toml` (Linux/macOS)
3. `%APPDATA%/flex-sh/config.toml` (Windows)
4. `config.toml` in current directory

### Quick Setup

```bash
# Copy the beautiful example config
cp flex-sh-config.toml ~/.config/flex-sh/config.toml

# Or customize the comprehensive example
cp example-config.toml ~/.config/flex-sh/config.toml
```

### Configuration Sections

#### 🎨 **Prompt Configuration**
```toml
[prompt]
format = "┌─[{user}@{hostname}]─[{cwd}]\\n└─❯ "
show_git = true
show_time = false
show_exit_code = true
```

#### 🌈 **Colors & Themes**
```toml
[colors]
enabled = true
scheme = "monokai"  # dark, light, monokai, default
command_color = "#50FA7B"
error_color = "#FF5555"
```

#### 📚 **History Management**
```toml
[history]
max_entries = 50000
ignore_duplicates = true
ignore_space_prefixed = true  # Commands starting with space won't be saved
```

#### 🔗 **Aliases**
```toml
[aliases]
# Git shortcuts
"gs" = "git status"
"ga" = "git add"
"gc" = "git commit -m"
"gp" = "git push"

# Navigation
".." = "cd .."
"~" = "cd ~"

# Development
"serve" = "python -m http.server 8000"
```

---

## 🔤 Tab Completion

Flex-SH features intelligent tab completion that works seamlessly across all scenarios:

### **Command Completion**
- **Built-in Commands** - `cd`, `ls`, `pwd`, `history`, etc.
- **System Programs** - All executables in your PATH
- **Smart Filtering** - Only relevant matches shown

### **Path Completion**
- **Simple Paths** - `./src/` → shows files in src directory
- **Complex Relative Paths** - `../../../project/src/` → full navigation support
- **Home Directory** - `~/` → expands to your home directory
- **Cross-Platform** - Works identically on Windows, Linux, and macOS

### **Advanced Features**
- **Case-Insensitive** - Works regardless of case (configurable)
- **Fuzzy Matching** - Smart partial matching
- **Directory First** - Directories shown before files
- **Hidden File Support** - Show/hide dotfiles as needed

---

## 🛠️ Built-in Commands

Flex-SH includes a comprehensive set of built-in commands:

| Command | Description | Example |
|---------|-------------|---------|
| `cd` | Change directory with tilde expansion | `cd ~/projects` |
| `ls` | List directory contents with colors | `ls -la` |
| `pwd` | Print working directory | `pwd` |
| `echo` | Print text with color support | `echo "Hello World"` |
| `history` | Command history management | `history 10` |
| `alias` | Create command shortcuts | `alias ll='ls -la'` |
| `env` | Environment variable management | `env PATH` |
| `which` | Find executable location | `which python` |
| `help` | Show available commands | `help` |
| `clear` | Clear terminal screen | `clear` |
| `exit` | Exit the shell | `exit` |

---

## 🔧 Advanced Features

### **Pipeline Support**
```bash
ls -la | grep ".rs" | wc -l
cat file.txt | sort | uniq > output.txt
```

### **Background Processes**
```bash
long-running-command &
python server.py &
```

### **Redirection**
```bash
command > output.txt      # Redirect stdout
command >> output.txt     # Append to file
command < input.txt       # Redirect stdin
command 2> errors.txt     # Redirect stderr
```

### **Environment Variables**
```bash
export MY_VAR=value
echo $MY_VAR
```

### **Command History**
```bash
history           # Show command history
history 20        # Show last 20 commands
Ctrl+R            # Reverse search history
```

---

## 🎯 Use Cases

### **Developers**
- **Git Workflow** - Integrated git shortcuts and branch display
- **Project Navigation** - Smart directory completion and bookmarks
- **Build Tools** - Aliases for common build commands and scripts
- **Multi-Platform** - Consistent experience across dev environments

### **System Administrators**
- **Server Management** - Remote-friendly prompts and efficient navigation
- **Batch Operations** - Powerful pipeline and redirection support
- **Process Management** - Background job control and monitoring
- **Security** - Sensitive command filtering and audit trails

### **Power Users**
- **Productivity** - Extensive customization and automation options
- **Aesthetics** - Beautiful, informative interface
- **Performance** - Fast execution and intelligent caching
- **Flexibility** - Adapt the shell to your exact workflow

---

## 📁 Project Structure

```
Flex-SH/
├── src/
│   ├── main.rs              # Entry point and CLI handling
│   ├── cli.rs               # Command-line argument parsing
│   ├── core/
│   │   ├── shell.rs         # Main shell implementation
│   │   ├── parser.rs        # Command parsing and tokenization
│   │   ├── executor.rs      # Command execution engine
│   │   └── history.rs       # Command history management
│   ├── builtins/            # Built-in commands
│   │   ├── cd.rs
│   │   ├── ls.rs
│   │   └── ...
│   ├── config/              # Configuration system
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── terminal/            # Terminal interface
│   │   ├── mod.rs
│   │   └── colors.rs
│   └── utils/               # Utility functions
├── config.toml              # Active configuration
├── flex-sh-config.toml      # Beautiful example config
├── example-config.toml      # Comprehensive config template
├── CONFIG.md               # Detailed configuration guide
└── README.md               # This file
```

---

## 🔧 Configuration Examples

### **Minimal Setup**
```toml
[prompt]
format = "{user}@{hostname}:{cwd}$ "

[colors]
enabled = true

[aliases]
ll = "ls -la"
".." = "cd .."
```

### **Developer Setup**
```toml
[prompt]
format = "┌─[{user}@{hostname}]─[{cwd}]\\n└─❯ "
show_git = true

[colors]
enabled = true
scheme = "monokai"

[aliases]
# Git workflow
"gs" = "git status"
"ga" = "git add"
"gc" = "git commit -m"
"gp" = "git push"
"gpl" = "git pull"

# Development
"serve" = "python -m http.server 8000"
"build" = "cargo build --release"
"test" = "cargo test"

[environment]
"EDITOR" = "code"
"RUST_BACKTRACE" = "1"
```

### **System Admin Setup**
```toml
[prompt]
format = "[{user}@{hostname} {cwd}]# "
show_exit_code = true

[history]
max_entries = 100000
ignore_space_prefixed = true

[aliases]
# System monitoring
"ps" = "ps aux"
"mem" = "free -h"
"disk" = "df -h"
"top" = "htop"

# Network
"ports" = "netstat -tlnp"
"ips" = "ip addr show"
```

---

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **🐛 Report Bugs** - Open an issue with details and reproduction steps
2. **💡 Suggest Features** - Share ideas for new functionality
3. **🔧 Submit PRs** - Fix bugs or implement new features
4. **📖 Improve Docs** - Help make documentation clearer
5. **🎨 Themes** - Create new color schemes and prompt designs

### Development Setup

```bash
# Clone and build
git clone https://github.com/yourusername/Flex-SH.git
cd Flex-SH
cargo build

# Run tests
cargo test

# Check code style
cargo fmt
cargo clippy
```

---

## 📜 License

Flex-SH is released under the MIT License. See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- **Rust Community** - For the excellent ecosystem and tools
- **Rustyline** - For readline functionality and tab completion
- **Tokio** - For async runtime and process management
- **Clap** - For command-line argument parsing
- **Serde** - For configuration serialization

---

## 📞 Support

- **📖 Documentation** - See [CONFIG.md](CONFIG.md) for detailed configuration
- **🐛 Issues** - Report bugs on GitHub Issues
- **💬 Discussions** - Join GitHub Discussions for questions and ideas
- **📧 Contact** - Reach out to the maintainers

---

**Made with ❤️ and ☕ by the Flex-SH team**

*Transform your command line experience with Flex-SH - where power meets beauty!*