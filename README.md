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
- **GUI Framework**: egui
- **Git Integration**: git2-rs
- **Build System**: Cargo

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

- **Phase 1**: Project Bootstrap ✓ (Current)
- **Phase 2**: Dependency Setup
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

**Current Phase**: Phase 1 - Project Bootstrap  
**Status**: ✅ Complete  
**Next Phase**: Phase 2 - Dependency Setup

---

*Built with ❤️ and Rust for the developer community*