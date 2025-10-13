---
experimental:
  v2_templates: true
---

# Component syntax

Docapella components use an [MDX](https://mdxjs.com/)-inspired syntax:

```html
<Card href="/features">
  **Hello, world**

  You can nest Markdown inside components
</Card>
```

## Tags

If you are familiar with HTML, Docapella's component syntax will feel very familiar.

All tags start with a capital letter. This is what distinguishes them from regular HTML tags.

Each tag has an _**opening tag**_ (`<Card>`), and an associated _**closing tag**_ (`<Card/>`). In between these tags you add optionally include content that will be rendered inside the component. In the above example, there are two paragraphs: `**Hello, world**`, and `You can nest Markdown inside components`.

### Validation

Importantly, Docapella **validates** your document structure. If you mismatch tags, you will get an error.

This input...

```html title="A component with mismatched tags"
<Card> Content </Box>
```

...will give you an error:

```plain title="Error message for a component with mismatched tags"
Unexpected closing tag `</Box>`, expected corresponding closing tag for `<Card>`

    1 │ <Card> Content </Box>
        ▲              ▲
        │              ╵
        └─ Opening tag
                       ╷
                       └─ Expected close tag
```

### HTML tags

You can also use regular HTML tags like component tags:

```html title="HTML with nested Markdown"
<div class="custom-class">**Nesting markdown** inside HTML Is allowed!</div>
```


## Attributes

Each component supports a set of **_attributes_**, which can change either the layout or behavior of the component.

For example, the [Button](./button.md) component accepts a `href` attribute, which defines the link the user will be taken to once they click the button:

```html
<Button href="https://www.example.com">Click me!</Button>
```
