# Image component

The `<Image>` component is similar to a standard Markdown image component, such as `[alt text](/url.png)`, but adds the ability for you to set a different image in light mode and dark mode.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex justify="center">
        <Image src="/_assets/components/image/light.png" src_dark="/_assets/components/image/dark.png" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Image component"
    <Image src="/_assets/components/image/light.png" src_dark="/_assets/components/image/dark.png" />
    ```
  </Tab>
</Tabs>

## Attributes

The `<Image>` component can be customized using the following attributes.

### Source

The `src` attribute specifies the default image to display. If referring to an image in your Docapella project, but be an absolute path to your `/_assets` directory, such as `/_assets/cat.jpg`.

This is a **required** attribute.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex justify="center" pad="2">
        <Image src="/_assets/doctave-logo.svg" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Image component"
    // [!code word:src:1]
    <Image src="/_assets/cat.jpg">
    ```
  </Tab>
</Tabs>

### Dark mode image

You can also specify a dark mode image using the `src_dark` attribute. This is useful for displaying a different image in dark mode and light mode.


<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex justify="center" pad="2">
        <Image src="/_assets/doctave-logo.svg" src_dark="/_assets/doctave-logo-dark.svg" />
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Image component"
    // [!code word:src_dark:1]
    <Image src="/_assets/cat.jpg" src_dark="/_assets/cat-dark.jpg">
    ```
  </Tab>
</Tabs>

### Alt text

You can specify an alt text for the image using the `alt` attribute. It's highly recommended to add alt texts to your images in order to improve the accessibility and SEO of your documentation.

<Tabs>
  <Tab title="Code">
    ```html title="Image component"
    // [!code word:alt:1]
    <Image src="/_assets/cat.jpg" alt="A cute cat">
    ```
  </Tab>
</Tabs>

