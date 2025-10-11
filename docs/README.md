# Docapella

Docapella is a static site generator for technical documentation. It's opinionated and designed to be easy to use, while still being flexible enough to handle most use cases.

It comes with built-in support for, OpenAPI 3.0 specifications, tabs and subtabs, search, and and extendable component system.

## Installation

<Callout type="warning">
    Currently, Docapella is only available via [Cargo](https://doc.rust-lang.org/cargo/). Other installation methods are coming soon.
</Callout>

First install Rust. You can find instructions [here](https://www.rust-lang.org/tools/install).

Then install Docapella:

```bash
cargo install --git https://github.com/Doctave/docapella.git
```

## Quick start

<Steps>
    <Step title="Create a a folder for your project">
        ```bash
        mkdir my-project
        cd my-project
        ```
    </Step>

    <Step title="Initialize the project">
        ```bash
        docapella init
        ```
    </Step>

    <Step title="Start the development server">
        ```bash
        docapella dev
        ```
    </Step>

    <Step title="Open your project in your browser">
        Open [localhost:8080](http://localhost:8080) in your browser to preview your documentation.
    </Step>
</Steps>


## Documentation

For more information, see the [documentation](https://docapella.com).
