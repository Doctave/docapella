---
experimental:
  v2_templates: true
---

# Fragment component

The `Fragment` component is a "passthrough" component. It has no impact on the actual layout of your content, but can be used for showing or hiding content conditionally.

## Conditionally showing content

If you have some Markdown content you want to show based on the result of an `if={...}` statement, you can use a `Fragment` tag:

```html
<Fragment if="{false}"> This content will not be shown </Fragment>
```

## Rendering

The component will have no impact on your actual rendered layout - it's only a way to wrap other content and optionally apply conditionals.
