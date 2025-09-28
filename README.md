# 🌿 Twiggy

**Lightning-fast Git Visualization Tool**

Twiggy is a high-performance Git visualization tool built with Rust and egui, designed to make version control intuitive through visual branch management and drag-&-drop operations.

## 🚀 Mission

To democratize Git workflow visualization by providing a tool that's faster than GitKraken, simpler than SourceTree, and more visual than command line - without the bloat.

## ✨ Key Features

- ⚡ **Speed**: Sub-second startup, instant repository loading
- 🎯 **Simplicity**: Intuitive visual interface, minimal learning curve  
- 🔧 **Efficiency**: Drag-&-drop Git operations, keyboard shortcuts
- 💾 **Lightweight**: <10MB download, <100MB RAM usage

## 🛠️ Technology Stack

- **Language**: Rust (2021 edition)
- **GUI Framework**: egui 0.24 (immediate mode GUI)
- **Application Runtime**: eframe 0.24 (cross-platform app framework)
- **Git Integration**: git2 0.18 (libgit2 Rust bindings)
- **Async Runtime**: tokio 1.0 (async/await support)
- **Serialization**: serde 1.0 + serde_json (configuration handling)
- **Error Handling**: anyhow 1.0 + thiserror 1.0 (robust error management)
- **System Integration**: directories 5.0 (cross-platform paths)
- **Date/Time**: chrono 0.4 (datetime handling for commits)
- **Logging**: tracing 0.1 + tracing-subscriber 0.3 (structured logging)
- **Build System**: Cargo

## 📋 System Requirements

### Minimum Requirements
- **OS**: Windows 10+, macOS 10.15+, or Linux (glibc 2.17+)
- **RAM**: 512MB available memory
- **Storage**: 50MB free disk space
- **Git**: Git 2.0+ installed and accessible in PATH

### Recommended Requirements
- **OS**: Windows 11, macOS 12+, or modern Linux distribution
- **RAM**: 2GB+ available memory for large repositories
- **Storage**: 100MB+ free disk space
- **Git**: Git 2.30+ for optimal compatibility

### Development Requirements
- **Rust**: 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **System Libraries**: 
  - Windows: Visual Studio Build Tools or Visual Studio 2019+
  - macOS: Xcode Command Line Tools
  - Linux: build-essential, libssl-dev, pkg-config

## 🔧 Dependency Rationale

### GUI Framework (egui/eframe)
- **egui**: Provides immediate mode GUI with excellent performance and native feel
- **eframe**: Handles window management, event loops, and cross-platform compatibility
- **Benefits**: Native compilation ensures superior performance vs Electron alternatives
- **Cross-platform**: Unified codebase for Windows, macOS, and Linux

### Git Integration (git2)
- **Direct libgit2 bindings**: Comprehensive Git functionality without subprocess overhead
- **Type-safe interface**: Rust's type system eliminates common Git operation errors
- **Performance**: Direct memory access to Git internals for fast repository operations
- **Advanced features**: Full access to Git internals enables sophisticated visualization

### Utility Libraries
- **serde ecosystem**: Robust serialization for user preferences and configuration files
- **anyhow/thiserror**: Professional error handling with context and custom error types
- **tokio**: Async runtime enables responsive UI during long-running Git operations
- **chrono**: Precise datetime handling for commit timestamps and history visualization
- **directories**: Cross-platform access to user config and cache directories
- **tracing**: Structured logging for debugging and performance monitoring

## 📦 Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git (for repository operations)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/twiggy-dev/twiggy.git
cd twiggy

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## 🏃 Quick Start

```bash
# Check project health
cargo check

# Run in development mode
cargo run

# Build optimized release
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## 📁 Project Structure

```
twiggy/
├── Cargo.toml          # Project configuration
├── src/
│   └── main.rs         # Application entry point
├── .gitignore          # Version control exclusions
├── README.md           # Project documentation
└── .git/               # Git repository metadata
```

## 🔄 Development Phases

Twiggy follows a 45-phase micro-development approach:

- **Phase 1**: Project Bootstrap ✓ (Complete)
- **Phase 2**: Dependency Setup ✓ (Complete)
- **Phase 3**: Basic UI Framework
- **Phase 4**: Git Repository Integration
- **Phase 5**: Visual Branch Rendering
- ... (42 more phases)

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Install development dependencies
rustup component add clippy rustfmt

# Run development checks
cargo check
cargo clippy
cargo fmt --check
cargo test
```

## 📄 License

This project is licensed under MIT OR Apache-2.0 - see the LICENSE files for details.

## 🔗 Links

- **Repository**: https://github.com/twiggy-dev/twiggy
- **Homepage**: https://twiggy-git.dev
- **Issues**: https://github.com/twiggy-dev/twiggy/issues

## 📊 Project Status

**Current Phase**: Phase 2 - Dependency Setup  
**Status**: ✅ Complete  
**Next Phase**: Phase 3 - Basic UI Framework

---

*Built with ❤️ and Rust for the developer community*