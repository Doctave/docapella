# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Docapella is a documentation generator written in Rust. It consists of a Rust workspace with three main crates:

- **docapella** (`crates/docapella/`): The main CLI application with commands for `init`, `build`, and `dev`
- **libdoctave** (`crates/libdoctave/`): The core library that handles documentation processing, markdown rendering, navigation building, and HTML template generation
- **openapi_parser** (`crates/openapi_parser/`): OpenAPI specification parsing utilities

The project is in the process of being migrated from the dynamic Doctave.com platform into a static site generator, with a more limited scope of features.

Namely, old projects on Doctave v1 will not be supported, but instead only v2 projects will work. Features involving servers like reader feedback, built-in analytics, and custom hosting of multiple versions won't be directly supported either.

The rendering engine is being migrated from the old Astro/React based system to a new Rust based system using Minijinja and Alpine.js.

## Architecture

### Core Components

- **Project Structure**: Projects are configured via `docapella.yaml` or `doctave.yaml` files (both formats supported)
- **Markdown Processing**: Uses `markdown-rs` for parsing with custom extensions and AST generation
- **Template System**: Uses minijinja for HTML templating with embedded templates in `src/templates/`
- **Navigation**: Supports both automatic (file hierarchy) and explicit navigation via YAML configuration
- **OpenAPI Support**: Built-in OpenAPI specification rendering with custom styling
- **Theming**: Color schemes, logos, and customizable CSS with built-in dark mode support

### Key Libraries Used

- `clap`: CLI argument parsing
- `serde` + `serde_yaml` + `serde_json`: Configuration and data serialization
- `minijinja`: Template engine for HTML generation
- `markdown-rs`: Markdown parsing and rendering
- `walkdir`: File system traversal
- `liquid`: Additional templating support
- `lightningcss`: CSS processing

## Development Commands

### Building
```bash
cargo build                    # Build all crates
cargo build --release          # Release build
```

### Testing
```bash
cargo test                     # Run all tests
cargo test --workspace         # Run tests across all workspace members
```

### Code Quality
```bash
cargo check                    # Fast syntax and type checking
cargo clippy                   # Linting (configured in clippy.toml)
cargo fmt                      # Format code
```

### Benchmarking
```bash
cargo bench                    # Run benchmarks (configured for libdoctave)
```

### Running the CLI
```bash
cargo run -- init             # Initialize new project
cargo run -- build            # Build documentation
cargo run -- dev              # Development server (not yet implemented)
```

## Project Configuration

Projects use YAML configuration files (`docapella.yaml` or `doctave.yaml`) with settings for:
- Theme configuration (colors, logos, dark mode)
- Navigation structure (tabs, subtabs, external links)
- Header and footer customization
- OpenAPI specification integration
- Content organization

## Template System

Templates are embedded in the binary and located in `crates/libdoctave/src/templates/`:
- HTML components in `components/`
- CSS files for styling
- Built-in support for OpenAPI documentation rendering
- Responsive design with mobile support

## Testing Strategy

The project includes:
- Unit tests throughout the codebase
- Integration tests in `tests/` directories
- Benchmarks for performance-critical components (markdown parsing, large OpenAPI specs)
- Example projects for testing different configurations

## File Structure Notes

- `test-area/`: Contains a test project for development
- `examples/`: Various example projects demonstrating features
- `boilerplate_project/`: Template files for new project initialization
- `icon_sets/`: SVG icon collections (Lucide icons)
- `bindings/`: TypeScript type definitions for data structures
