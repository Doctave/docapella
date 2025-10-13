# Card component

A `<Card>` is a way to highlight and group content. They can be used effectively with [Grids](/components/grid) to layout blocks of information.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Card>
        Basic content with some **markdown**
      </Card>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Card component"
    <Card>
      Basic content with some **markdown**
    </Card>
    ```
  </Tab>
</Tabs>

## Attributes

You can customize the `<Card>` component using the following attributes.

### Clickable cards

You can turn the whole card into a clickable link by specifying the `href=".."` attribute.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Card href="https://www.example.com">
        Click me!
      </Card>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Clickable Card component"
    // [!code word:href:1]
    <Card href="https://www.example.com">
      Click me!
    </Card>
    ```
  </Tab>
</Tabs>

### Padding

The `pad` attribute sets the padding around the content within the callout. It accepts values from 0 to 5, where 0 indicates no padding, and 5 indicates the highest padding level. The default padding is 3.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Card pad="5">
        This is a card with a lot of padding.
      </Card>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Card component with padding"
    // [!code word:pad:1]
    <Card pad="5">
        This is a card with a lot of padding.
    </Card>
    ```
  </Tab>
</Tabs>

### Max width

You can vary the maximum width of the card with the `max_width=".."` attribute.

The width must be `xs`, `md`, `lg`, `xl`, or `full` (default).

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Card max_width="md">
        Card with `md` max width
      </Card>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Card component with a max width"
    // [!code word:max_width:1]
    <Card max_width="md">
      Card with `md` max width
    </Card>
    ```
  </Tab>
</Tabs>

