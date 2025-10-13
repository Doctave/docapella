# Custom reusable components

While Docapella comes with lots of useful reusable components out of the box, you can also create your own! This is incredibly useful if you want to implement a specialised widget that Docapella doesn't provide out of the box.

## Tutorial

Let's walk through creating a custom "Card" component.

All custom components live under the `_components` directory at the root of your project (the same directory your `doctave.yaml` file is in).

### Creating the component

Create the following file under `_components/custom-card.md`:

```html
---
attributes:
  - title: heading
    required: true
---

<Box pad="3" class="custom-card-class">
  **{ @heading }**

  <Slot />
</Box>
```

Don't worry about the details yet - let's first learn to invoke this component.

### Rendering the component

Now, in your documentation, you can render the component with the following syntax:

```html
<Component.CustomCard title="My title">
  The content that goes _inside_ the card
</Component.CustomCard>
```

Next, let's walk through this example to understand how components work in detail.

## Attributes

Each component can define attributes that they will receive in its frontmatter. You can use these attributes as variables inside your component template.

Attributes **must** be declared in order to be used.

In the example above, we're defining a single attribute: `heading`:

```
---
attributes:
  - title: heading
    required: true
---
```

All attributes are put under the `attributes` field as a list of YAML objects. The attribute **must** define a `title` field, which **must** be a valid identifier.

Optionally, the attribute can be marked as `required`, in which case the caller gets an error if a value hasn't been provided for the attribute. Attributes that aren't `required` will default to `null` if no value is given.

## Slot

It's common to want to let the callers of components to "inject" Markdown content inside your components. This is where the special `<Slot />` tag comes in.

Let's look at the above component again:

```html
<Box pad="3" class="custom-card-class">
  **{ @heading }**

  <Slot />
</Box>
```

When this component is called, the `<Slot />` component will be _replaced_ with what the caller includes between the open and close tag of the component.

Let's look at another example. I want to add some body text and a custom `Button` into the card:

```html
<!-- This will get turned into... -->
<Component.CustomCard title="My title">
  The content that goes _inside_ the card

  <button href="https://www.example.com">Sign Up</button>
</Component.CustomCard>
```

Here, the `<Slot />` tag will be replaced with everything inside the open and close tag:

```html
<!-- <Slot /> replaced with the following: -->
The content that goes _inside_ the card

<button href="https://www.example.com">Sign Up</button>
```

And the final output will be rendered as:

```html
<Box pad="3" class="custom-card-class">
  **My title** The content that goes _inside_ the card

  <button href="https://www.example.com">Sign Up</button>
</Box>
```

## Component names

Components derive their names from their path in your project. Every custom component is always prefixed by `Component` in order to prevent clashes with Docapella's built-in components.

All component names will be converted to [camel case](https://en.wikipedia.org/wiki/Camel_case).

Here are some examples of the rules:

| Path                            | Name                       |
| ------------------------------- | -------------------------- |
| `_components/example.md`        | `Component.Example`        |
| `_components/example-button.md` | `Component.ExampleButton`  |
| `_components/example_button.md` | `Component.ExampleButton`  |
| `_components/button/primary.md` | `Component.Button.Primary` |

**NOTE:** Component names may conflict! Two different paths may map to the same component name. In this case, any of the conflicting components may be chosen randomly.
