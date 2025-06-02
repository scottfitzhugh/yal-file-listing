# Rust File Lister ⚡

A lightning-fast file listing utility written in Rust, inspired by `ls`, `lsd`, and `exa`. Display directory contents in a beautiful grid with file type icons, permissions, and color-coded output.

## Description

This high-performance file listing tool provides:
- **Lightning-fast directory traversal** optimized for speed
- **Beautiful grid layout** that adapts to your terminal width
- **File type icons** using Nerd Fonts glyphs for instant visual recognition ( for Rust,  for directories, etc.)
- **Octal permissions display** (644, 755, etc.)
- **Owner and group information** with color coding
- **Intelligent sorting** (directories first, then alphabetical)
- **Cross-platform terminal support** with colors

## Prerequisites

- Rust programming language (latest stable version recommended)
- Cargo (comes with Rust installation)
- Unix-like system (Linux, macOS) for file permissions
- **Nerd Fonts** installed for proper icon display (recommended: FiraCode Nerd Font, JetBrains Mono Nerd Font)

## Installation

1. Clone or download this repository
2. Navigate to the project directory

## Usage

To compile and run the file lister:

```bash
cargo run
```

To compile for production use:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/rusttest`

## Features

### 🎨 Visual Elements
- **File Type Icons**: Instant recognition with Nerd Fonts glyphs
  -  Rust files (.rs)
  -  Python files (.py) 
  -  JavaScript/TypeScript (.js, .ts)
  -  Directories
  -  Markdown files (.md)
  -  Images (.jpg, .png, .gif, etc.)
  - And many more!

### 🎯 Performance
- **Zero-allocation file type detection**
- **Efficient memory usage**
- **Adaptive grid layout**
- **Graceful error handling**

### 🌈 Color Coding
- **Blue Bold**: Directory names
- **Yellow**: File permissions  
- **Green**: Owner names
- **Cyan**: Group names

## Example Output

```
 /home/user/projects/rusttest (6 items)

 755 user user src/            644 user user main.rs        
 644 user user Cargo.toml     644 user user README.md      
 644 user user project-plan.md    755 user user hello_world
```

## Project Structure

- `src/main.rs` - Main application source code with file listing logic
- `Cargo.toml` - Project configuration and dependencies
- `project-plan.md` - Detailed project planning and documentation
- `README.md` - This file

## Dependencies

- **users (0.11)** - For resolving user/group names from UIDs/GIDs
- **crossterm (0.27)** - For cross-platform terminal colors and terminal size detection

## Building from Source

```bash
# Debug build (fast compilation)
cargo build

# Release build (optimized for performance)
cargo build --release

# Run tests (when available)
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Benchmarks

This utility is designed for speed:
- Handles large directories efficiently
- Minimal memory allocations
- Fast file type detection
- Optimized sorting algorithms

## License

This project is provided as-is for educational and demonstration purposes. 