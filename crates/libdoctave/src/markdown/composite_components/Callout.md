---
attributes:
  - title: type
    required: false
    default: "info"
    validation:
      is_a: text
      is_one_of:
        - "info"
        - "success"
        - "warning"
        - "error"
  - title: pad
    required: false
    default: 2
    validation:
      is_a: number
      is_one_of:
        - 0
        - 1
        - 2
        - 3
        - 4
        - 5
---

<Box class={"d-callout d-callout-" | append(@type)} pad={@pad} >
  <Slot />
</Box>
