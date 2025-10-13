# Filters

Filters are helpers you can call to transform values in your documentation inside expressions.

This is the list of filters supported by Doctave.

---

### `capitalize`

Capitalizes a string

##### Usage

```elixir
"teddy bear" | capitalize  # => "Teddy bear"
```

```elixir
capitalize("teddy bear") # => "Teddy bear"
```

##### Arguments

| Position | Description          | type     | required |
| -------- | -------------------- | -------- | -------- |
| 1        | String to capitalize | `string` | true     |

---

### `append`

Appends one string to another.

##### Usage

```elixir
"teddy" | append(" bear")  # => "teddy bear"
```

```elixir
append("teddy", " bear")  # => "teddy bear"
```

##### Arguments

| Position | Description      | type     | required |
| -------- | ---------------- | -------- | -------- |
| 1        | Original string  | `string` | true     |
| 2        | String to append | `string` | true     |
