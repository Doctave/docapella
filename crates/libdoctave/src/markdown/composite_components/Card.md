---
attributes:
  - title: href
    required: false
  - title: pad
    required: false
    default: 3
    validation:
      is_a: number
      is_one_of:
        - 0
        - 1
        - 2
        - 3
        - 4
        - 5
  - title: max_width
    required: false
    default: "full"
    validation:
      is_a: text
      is_one_of:
        - "sm"
        - "md"
        - "lg"
        - "xl"
        - "full"
  - title: class
    required: false
    default: ""
    validation:
      is_a: text
---

<a class="d-card-link" if={@href} href={@href}>
  <Box class={append("d-card d-hover ", @class)} pad={@pad} max_width={@max_width}>
    <Slot />
  </Box>
</a>
<Box else class={append("d-card ", @class)} pad={@pad} max_width={@max_width}>
  <Slot />
</Box>
