---
experimental:
  v2_templates: true
---

# Button Component

A `<Button>` acts just like a link, but is styled like a button. Also supports the `target="_blank"` attribute for opening links in new tabs.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex pad="2">
        <Button href="https://www.example.com">Example</Button>
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Button component"
    <Button href="https://www.example.com">My button</Button>
    ```
  </Tab>
</Tabs>

## Attributes

The `<Button>` component can be customized using the following attributes.

### Size

You can adjust the button's size with the `size=".."` attribute. 

The size must be `sm`, `md` (default), or `lg`.


<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex pad="2" gap="3" justify="between" align="center">
        <Box>
          <Button size="sm" href="https://www.example.com">Small button</Button>
        </Box>

        <Box>
          <Button size="md" href="https://www.example.com">Medium button</Button>
        </Box>

        <Box>
          <Button size="lg" href="https://www.example.com">Large button</Button>
        </Box>
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Specifying the size of the button"
    // [!code word:size:1]
    <Button size="lg" href="/">My button</Button>
    ```
  </Tab>
</Tabs>

### Variant

You can change the style of the button with the `variant=".."` attribute.

The variant must be `primary` (default), `secondary`, 'outline', or `ghost`.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex pad="2" gap="2" align="center" justify="between">
        <Box>
          <Button variant="primary" href="https://www.example.com">Primary button</Button>
        </Box>

        <Box>
          <Button variant="secondary" href="https://www.example.com">Secondary button</Button>
        </Box>

        <Box>
          <Button variant="outline" href="https://www.example.com">Outline button</Button>
        </Box>

        <Box>
          <Button variant="ghost" href="https://www.example.com">Ghost button</Button>
        </Box>
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Changing the style of the button"
    // [!code word:variant:1]
    <Button variant="secondary" href="/">My button</Button>
    ```
  </Tab>
</Tabs>


### Width

You can make the button full width by specifying `width="full"`.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex pad="2">
        <Button width="full" size="lg" href="https://www.example.com">My button</Button>
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Making the button full width"
    // [!code word:width:1]
    <Button width="full" href="/">My button</Button>
    ```
  </Tab>
</Tabs>

### Target

If you want your button to open the link in a new tab, you can set `target="_blank"`.

<Tabs>
  <Tab title="Code">
  ```html title="Opening the link in a new tab"
  // [!code word:target:1]
  <Button target="_blank" href="/">My button</Button>
  ```
  </Tab>
</Tabs>

### Class

You can add a custom CSS class to the button using the `class` attribute. This is useful for applying custom styles to the button.

<Tabs>
  <Tab title="Code">
    ```html title="Custom class"
    // [!code word:class:1]
    <Button class="my-custom-css">My button</Button>
    ```
  </Tab>
</Tabs>
