---
experimental:
  v2_templates: true
---

# Flex component

The `Flex` component is a building block for making more complex layouts without resorting to custom CSS.

The component is modelled after the CSS [flexbox](https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_flexible_box_layout/Basic_concepts_of_flexbox) concept: which is a powerful way to layout elements.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex gap="2">
        <Box class="demo-blob" />

        <Box class="demo-blob" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Flex can be used to align items horizontally"
    <Flex>
      ...
    </Flex>
    ```
  </Tab>
</Tabs>


## Attributes

The `<Flex>` component can be customized using the following attributes.

### Gap

You can increase spacing between children with the `gap` attribute. It must be a value between 0 and 5 inclusive. The default gap is 0.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <div style="background-color: var(--accent-4); width: fit-content;">
        <Flex gap="4">
          <Box class="demo-blob" />
          <Box class="demo-blob" />
          <Box class="demo-blob" />
        </Flex>
      </div>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Flex with gap specified"
    // [!code word:gap:1]
    <Flex gap="4">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>


### Justify

You can use the `justify` attribute to place items according to each other in the `Flex` component.

Must be one of `start`, `end`, `center`, `between`. Defaults to `start`.

<Tabs>
  <Tab title="Justify start">
    <Component.ComponentDemo>
      <Flex justify="start" gap="2">
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Center aligning content with center justification"
    // [!code word:justify:1]
    <Flex justify="start">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

<Tabs>
  <Tab title="Justify end">
    <Component.ComponentDemo>
      <Flex justify="end" gap="2">
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Center aligning content with center justification"
    // [!code word:justify:1]
    <Flex justify="end">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

<Tabs>
  <Tab title="Justify center">
    <Component.ComponentDemo>
      <Flex justify="center" gap="2">
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Center aligning content with center justification"
    // [!code word:justify:1]
    <Flex justify="center">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

<Tabs>
  <Tab title="Justify between">
    <Component.ComponentDemo>
      <Flex justify="between" gap="2">
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Center aligning content with center justification"
    // [!code word:justify:1]
    <Flex justify="between">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

### Wrapping

You can use the `wrap` attribute to specify how elements should wrap inside the Flex container.

Must be one of `wrap`, `nowrap`, `wrapreverse`. Defaults to `nowrap`.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex wrap="wrap" gap="2">
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Wrapping flex content with wrap"
    // [!code word:wrap:1]
    <Flex wrap="wrap">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

### Align

You can use the `align` attribute to align items horizontally (or vertically, if in the Flex direction is `column`) in the `Flex` component. This is equivalent to the flexbox `align-items` attribute.

Must be one of `start`, `end`, `center`, `stretch`, `baseline`. Defaults to `start`.

<Tabs>
  <Tab title="Code">
    ```jsx title="Center aligning content with center justification"
    // [!code word:align:1]
    <Flex align="center">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

### Height

You can tell your `Flex` element to consume the full height of its container with the `height` attribute.

<Tabs>
  <Tab title="Code">
    ```jsx title="Height of flex container"
    // [!code word:height:1]
    <Flex height="full">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

Must be one of `auto` or `full`. Defaults to `auto`.

### Direction

The `dir` attribute can be used to change the direction of the flex container from horizontal to vertical.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex dir="column" gap="2">
        <Box class="demo-blob" />
        <Box class="demo-blob" />
        <Box class="demo-blob" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Vertical flex container"
    // [!code word:dir:1]
    <Flex dir="column">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>

### Class

You can pass a custom class to your Flex element with the `class` attribute:

<Tabs>
  <Tab title="Code">
    ```jsx title="Custom class"
    // [!code word:class:1]
    <Flex class="custom-flex">
      ...
    </Flex>
    ```
  </Tab>
</Tabs>
