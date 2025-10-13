# Callout Component

The `<Callout>` component is a versatile tool for drawing attention to important information within a user interface. It's particularly useful for displaying messages such as warnings, informative notes, success confirmations, or error alerts. 

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Callout type="success">
        This is an success message.
      </Callout>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Callout component"
    <Callout type="success">
        This is an success message.
    </Callout>
    ```
  </Tab>
</Tabs>


## Attributes

The `<Callout>` component has the following attributes:

### Type

The `type=".."` attribute defines the color and semantics of callout. It can be one of the following:

- `warning`: Indicates a warning message.
- `info`: Represents an informative message.
- `success`: Indicates a successful action or confirmation.
- `error`: Represents an error or failure.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Callout type="warning">
        Before proceeding, ensure you have completed the previous steps
      </Callout>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Callout component"
    <Callout type="warning">
        Before proceeding, ensure you have completed the previous steps
    </Callout>
    ```
  </Tab>
</Tabs>

### Padding

The `pad` attribute sets the padding around the content within the callout. It accepts values from 0 to 5, where 0 indicates no padding, and 5 indicates the highest padding level. The default padding is 3.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Callout type="success" pad="5">
        This is positive message with a lot of padding.
      </Callout>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Callout component"
    <Callout type="success" pad="5">
        This is positive message with a lot of padding.
    </Callout>
    ```
  </Tab>
</Tabs>
