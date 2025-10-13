# Project structure

Let's take a look at the structure of an example Docapella project.

```plain title="Example project structure"
├── docapella.yaml
├── navigation.yaml
├── openapi.yaml
├── README.md
├── tutorials
│   └── getting-started.md
└── _assets
    ├── logo.svg
    └── logo-dark.svg
```

## Configuration

The `docapella.yaml` file is the main configuration file for your project. It contains settings for the theme, OpenAPI specifications, and more.

[Read the configuration reference](/configuration-reference.md) for more information.

## Navigation

The `navigation.yaml` file defines the structure of your project. It defines the left sidebar navigation, and can also be used to generate the navigation structure for your OpenAPI specifications.

Each tab and subtabs gets its own `navigation.yaml` file - this allows you to have multiple tabs with different navigation structures.

[Read the navigation reference](/navigation.md) for more information.

## OpenAPI specifications

Docapella supports OpenAPI 3.0 specifications. You can add your OpenAPI specifications to your project and Docapella will generate a navigation structure for them.

[Read the OpenAPI documentation](/openapi.md) for more information.

## Markdown files

Markdown files are the main content of your project. Each Markdown file maps to a page in your documentation, and its location in your project defines the URL it will be accessible with.

For example, if you have a Markdown file at `tutorials/getting-started.md`, it will be accessible at `tutorials/getting-started`.

`README.md` is the only exception to this rule, and they work exactly as `index.html` files. The root `README.md` will map to the root URL `/` and a `subdir/README.md` is accessible at `/subdir`.

## Assets

Assets are files like images that are used in your Markdown files and they live in the `_assets` folder.

You can reference them in your Markdown files using the `![alt text](/_assets/image.png)` syntax.

Read more about [assets here](/assets.md).

