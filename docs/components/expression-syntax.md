# Expression Syntax

You can use expressions to use dynamic content in your docs, or conditionally show specific content.

Here is a simple example:

```
{ "bob" | capitalize }
```

This would render: `Bob`.

**NOTE**: Dynamic expressions and components aren't supported inside code blocks, in order to prevent clashes with any languages with similar syntax.

<Callout type="info">
  With the migration from Doctave to Docapella, expressions will be able to be more flexible, since we don't have to worry about safety when interpreting expressions on Doctave's servers. Expect updates and new capabilities in this area.
</Callout>

## Filters

Filters are ways for you to transform values inside expressions. You can call filters like functions in most programming languages:

```elixir
capitalize("docs")  # => "Docs"
```

But you can also use a pipeline syntax, where values flow from one filter to another in sequence:

```elixir
"docu" | capitalize | append("mentation") # => "Documentation"
```

[You can find the list of supported filters here ›](./filters.md)

## Types

Doctave's expression language supports the following basic types:

- Strings
- Integers
- Floats
- Boolean values
- Lists _(coming soon)_
- Objects _(coming soon)_
- Null

### Truthiness

All values are `truthy`, except for `null` and `false`. This allows you to for example set a default value in case a variable is null:

```
{ @maybe_null || "Default value" }
```
