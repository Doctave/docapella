# OpenAPI documentation

Docapella supports OpenAPI 3.0 specifications. You can add your OpenAPI specifications to your project and Docapella will generate a navigation structure for them.

## Adding an OpenAPI specification

To add an OpenAPI specification to your project, add your JSON or YAML file to the root of your project, and then add a reference to it in your `docapella.yaml` file:

```yaml title="docapella.yaml"
open_api:
  - spec_file: openapi.yaml
    uri_prefix: /api
```

What this does is tell Docapella to generate a navigation structure for the OpenAPI specification, and to serve the specification at the `/api` URL.

## OpenAPI documentation structure

Docapella will generate one page for each tag in your OpenAPI specification, as well as a page for the overview of the entire specification.

### Overview page

The overview page will show the current version of the specification, and will list all the server URLs that are available.

It will also include the top level `description` Markdown field from the specification, if it exists.

```yaml title="openapi.yaml overview description"
// [!code word:description:1]
openapi: 3.0.0
info:
  title: Nebularis Example API
  description: |
    This is an **example OpenAPI spec** for an imaginary cloud orchestration company.
```

### Tag pages

For each tag in your specification, Docapella will generate a page with the tag name. This page will show the description of the tag, and will list all the operations that are available in the tag.

You can customize the description of the tag page by adding a `tag.description` field to the tag object in your specification.

```yaml title="openapi.yaml"
tags:
  - name: Users
    description: Example tag description for a Users tag
```

## Navigation

Docapella can generate the left-side navigation structure for your OpenAPI specification, showing the operations and associated HTTP verbs.

```yaml title="navigation.yaml"
- heading: API Reference
  items:
  - open_api_spec: openapi.json
```

This will generate a link for each tag, and under each tag, a link for each operation.

### Filtering operations

If you want more control over which operations are shown in the navigation, you can use the `only` field to filter the operations.

This can be used to split the navigation into multiple sections, or to only show operations that are relevant for the section.

```yaml title="openapi.yaml"
- heading: Access Control API
  items:
  - open_api_spec: openapi.json
    only:         # <- Only render links for the following 2 OpenAPI tags:
      - Authentication
      - Authorization
```
