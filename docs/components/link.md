# Link component

The `<Link>` component works just like a normal Markdown link, but with a few additional useful features.

- You can set `target="_blank"` to open links in a new tab
- You can add a custom class with `class="custom class"`
- The link text can be arbitrary Markdown

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Link href="/">Click here</Link>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Link component"
    <Link href="/">Click here</Link>
    ```
  </Tab>
</Tabs>

## Attributes

The `<Link>` component can be customized using the following attributes.

### Custom class

You can add a custom class to your link with the `class` attribute:

<Tabs>
  <Tab title="Code">
    ```html title="Link component with a custom class"
    // [!code word:class:1]
    <Link href="/" class="custom-styles">Click here</Link>
    ```
  </Tab>
</Tabs>

### Opening links in a new tab

You can make the link open in a new tag with `target="_blank"`

<Tabs>
  <Tab title="Code">
    ```html title="Opening a link in a new tab"
    // [!code word:target:1]
    <Link href="/" target="_blank">Open in new tab</Link>
    ```
  </Tab>
</Tabs>

