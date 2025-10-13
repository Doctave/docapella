# Icon component

The `<Icon>` component allows you to access a set of pre-installed icon libraries and use them in your documentation.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex gap="5" justify="center" align="center">
        <Icon set="lucide" name="fingerprint" size="sm" variant="plain" />

        <Icon set="lucide" name="fingerprint" size="md" variant="plain" />

        <Icon set="lucide" name="fingerprint" size="lg" variant="plain" />

        <Icon set="lucide" name="fingerprint" size="xl" variant="plain" />

        <Icon set="devicon" name="typescript" size="sm" variant="boxed" color />

        <Icon set="devicon" name="typescript" size="md" variant="boxed" color />

        <Icon set="devicon" name="typescript" size="lg" variant="boxed" color />

        <Icon set="devicon" name="typescript" size="xl" variant="boxed" color />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Rendering an icon"
    <Icon set="devicon" name="typescript" size="lg" variant="boxed" color />
    ```
  </Tab>
</Tabs>

## Icon sets

The two icon sets supported are [Lucide](https://lucide.dev/) and [Devicon](https://devicon.dev/).

## Attributes

The `<Icon>` component accepts the following attributes.

### Set

The `set` attribute specifies the icon set to use. It must be either `lucide` or `devicon`.

This is a **required** attribute.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex justify="center">
        <Icon set="lucide" name="dog" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Rendering an icon"
    // [!code word:set:1]
    <Icon set="lucide" name="dog" />
    ```
  </Tab>
</Tabs>

### Name

The `name` attribute specifies the name of the icon to use within the chosen set.

This is a **required** attribute.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex justify="center">
        <Icon set="lucide" name="dog" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Rendering an icon"
    // [!code word:name:1]
    <Icon set="lucide" name="dog" />
    ```
  </Tab>
</Tabs>

### Color

Specifying the `color` attribute will fill the icon with your theme's accent color.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex justify="center">
        <Icon set="lucide" name="dog" color />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Rendering an icon"
    // [!code word:color:1]
    <Icon set="lucide" name="dog" color />
    ```
  </Tab>
</Tabs>

### Variant

You can also specify a `plain` or `boxed` variant. The latter will add some padding around your icon.

Defaults to `plain`. Can be combined with `color`.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex gap="4" justify="center">
      <Icon set="lucide" name="dog" variant="boxed" />

      <Icon set="lucide" name="dog" variant="boxed" color />
    </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Rendering an icon"
    // [!code word:variant:1]
    <Icon set="lucide" name="dog" variant="boxed" />
    ```
  </Tab>
</Tabs>

### Size

Finally, you can set a `size` of `sm`, `md`, `lg`, or `xl` (the default is `md`).

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex gap="5" pad="2" justify="center">
          <Icon set="devicon" name="typescript" size="sm" />

          <Icon set="devicon" name="typescript" size="md" />

          <Icon set="devicon" name="typescript" size="lg" />

          <Icon set="devicon" name="typescript" size="xl" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Rendering an icon"
    // [!code word:size:1]
    <Icon set="devicon" name="typescript" size="lg" />
    ```
  </Tab>
</Tabs>

### Class

You can add a custom CSS class to the icon using the `class` attribute. This is useful for applying custom styles to the icon.



<Tabs>
  <Tab title="Code">
    ```html title="Custom class"
    // [!code word:class:1]
    <Icon set="lucide" name="dog" class="my-custom-css">
    ```
  </Tab>
</Tabs>
