# Theme

While Docapella's overall structure is fairly fixed, you can customize the look and feel of your project by changing the theme.

## Colors

Docapella uses a 12-step color system similar to [Radix Colors](https://www.radix-ui.com/colors). All you have to do is provide your main theme color, and Docapella will generate your palette for you in both light and dark mode.

```yaml title="docapella.yaml · Colors"
theme:
  colors:
    # Your brand's main color
    accent: "#F76B15"
```

This will generate the following palette:

<Flex justify="center">
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-1)"> <small>1</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-2)"> <small>2</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-3)"> <small>3</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-3)"> <small>3</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-4)"> <small>4</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-5)"> <small>5</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-6)"> <small>6</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-7); color: var(--accent-1)"> <small>7</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-8); color: var(--accent-1)"> <small>8</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-9); color: var(--accent-1)"> <small>9</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-10); color: var(--accent-1)"> <small>10</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-11); color: var(--accent-1)"> <small>11</small> </div>
  <div style="text-align: center; width: 2rem; height: 2rem; background-color: var(--accent-12); color: var(--accent-1)"> <small>12</small> </div>
</Flex>

Try turning on dark mode to see the difference.

## Logo

You can specify a logo for your project. This will be shown in the header of your documentation.

```yaml title="docapella.yaml · Logo"
theme:
  logo:
    src: _assets/logo.svg
    src_dark: _assets/logo-dark.svg
```

Note how you can specify a dark mode logo by using `src_dark`. If none is provided, Docapella will fall back to the light mode logo.
