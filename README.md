Docapella, an opinionated Markdown and OpenAPI documentation generator 
======================================================================

Docapella is a static site generator that turns your Markdown and OpenAPI specifications into a documentation website.

## Status

This project is currently pre 1.0. Some features are still being worked on, and installation is only supported via Cargo.

Occasional breaking changes may occur, but we'll try to keep them to a minimum.

## Installation

First, install Rust. You can find instructions [here](https://www.rust-lang.org/tools/install).

Next, install Docapella:

```bash
cargo install --git https://github.com/Doctave/docapella.git
```

## Usage

The basic commands are:

### Creating a new project: `docapella init`

```bash
docapella init
```

This will create a `docapella.yaml` file in the current directory and a `README.md` file.

### Running the development server: `docapella dev`

```bash
docapella dev
```

This will start a local server and open your documentation in your browser.

The port defaults to 8080, but can be changed by passing the `--port` flag.

### Building the project: `docapella build`

```bash
docapella build
```

This will build the project and output the static files to the `_build` directory, which can be served with any static file server.

