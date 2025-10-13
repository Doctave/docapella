---
experimental:
  v2_templates: true
---

# Box component

A `<Box>` component is a way to group together Markdown content inside other components. For example, if you want to group together two paragraphs inside a `<Grid>` element, you can wrap them in a `<Box>`.

Under the hood, `<Box>` will be converted to a plain `<div>`.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Box>
        I am in a box!
      </Box>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Box component"
    <Box>
      I am in a box!
    </Box>
    ```
  </Tab>
</Tabs>

## Attributes

The `<Box>` component can be customized using the following attributes.

### Padding

You can adjust the padding inside the `<Box>` with the `p` attribute. It must be a value between 0 and 5 inclusive.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Box pad="5">
        I am in a box with a padding of 5!
      </Box>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Box component with a padding of 5"
    // [!code word:pad:1]
    <Box pad="5">
      I am in a box with a padding of 5!
    </Box>
    ```
  </Tab>
</Tabs>

### Height

You can tell your `Box` element to consume the full height of its container with the `height` attribute.

Must be one of `auto` or `full`. Defaults to `auto`.

<Tabs>
  <Tab title="Code">
    ```html title="Box component with a height of 100%"
    // [!code word:height:1]
    <Box height="full">
      I am in a box with a height of 100%
    </Box>
    ```
  </Tab>
</Tabs>
