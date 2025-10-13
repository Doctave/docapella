# Component Expression Syntax

You can use expressions to use dynamic content in your docs, or conditionally show specific content.
Expression syntax is primarily used with [user preferences](/contents/user-preferences.md).

Here is a simple example:

```
{ "bob" | capitalize }
```

This would render: `Bob`.

**NOTE**: Dynamic expressions and components aren't supported inside code blocks, in order to prevent clashes with any languages with similar syntax.

## Expressions in components

You can use expressions as arguments to components. The only change you have to make is instead of using the HTML-style attribute syntax (`key="value"`), you use curly braces instead of quotes for the value of the attribute (`key={ value }`):

For example, if you want to use a currently select user preference in a component, you can do that as follows:

```html
<Component.Example plan={ @user_preferences.plan } />
```

You can also use more complex expressions:

```html
<Card title={"Current plan: " | append(@user_preferences.plan) } />
```

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
