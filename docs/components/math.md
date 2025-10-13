---
title: Math notation
---

Math
====

Doctave supports mathematical equations via [KaTeX](https://katex.org) to enable rendering mathematical expressions in your documentation.

Visit the [KaTeX documentation](https://katex.org/docs/supported.html) for a list of supported functions and symbols.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      $$
      lambda = \frac{1}{n} \sum_{i=1}^n \left( x_i - \bar{x} \right)^2
      $$
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```latex title="Math notation"
    $$
    lambda = \frac{1}{n} \sum_{i=1}^n \left( x_i - \bar{x} \right)^2
    $$
    ```
  </Tab>
</Tabs>

<Callout type="warning">
  <Flex gap="1">
    <Icon set="lucide" name="triangle-alert" />

    Doctave does not validate the syntax of your mathematical expressions.
  </Flex>
</Callout>

## Inline syntax

To render inline math, wrap your mathematical expression with double dollar signs (`$$`). This renders the math expression within the flow of the text.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Flex>
        Some math $$(a - b)^2$$ here.
      </Flex>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```latex title="Inline math notation"
    Some math $$(a - b)^2$$ here.
    ```
  </Tab>
</Tabs>

## Block syntax

If you want to display a mathematical expression on its own line, wrap the expression with double dollar signs (`$$`) and place it on a new line.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      Here is some math:

      $$
      (a - b)^2
      $$

      What was some math!
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```latex title="Block math notation"
    Here is some math:

    $$
    (a - b)^2
    $$

    What was some math!
    ```
  </Tab>
</Tabs>


## Attributes

You can customize the math notation using the following attributes.


### Display Mode

[Display mode](https://katex.org/docs/options.html) is used for more complex equations that require additional formatting, such as alignment. To enable display mode, set the `display_mode` parameter to `true`.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      $$ display_mode=true
      \begin{equation}
      \begin{split}
      (a - b)^2 &= (a - b)(a - b) \\
      &= a(a - b) - b(a - b)      \\
      &= a^2 - ab - ba + b^2      \\
      &= a^2 - 2ab + b^2          \nonumber
      \end{split}
      \end{equation}
      $$
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```latex title="Math block with display mode enabled"
    $$ display_mode=true
    \begin{equation}
    \begin{split}
    (a - b)^2 &= (a - b)(a - b) \\
    &= a(a - b) - b(a - b)      \\
    &= a^2 - ab - ba + b^2      \\
    &= a^2 - 2ab + b^2          \nonumber
    \end{split}
    \end{equation}
    $$
    ```
  </Tab>
</Tabs>
