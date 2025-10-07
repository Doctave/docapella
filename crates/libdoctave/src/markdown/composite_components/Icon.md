---
attributes:
  - title: set
    required: true
    validation:
      is_a: text
      is_one_of:
        - devicon
        - lucide
  - title: name
    required: true
    validation:
      is_a: text
  - title: size
    default: "md"
    validation:
      is_a: text
      is_one_of:
        - sm
        - md
        - lg
        - xl
  - title: variant
    default: plain
    validation:
      is_a: text
      is_one_of:
        - plain
        - boxed
  - title: color
    default: false
    validation:
      is_a: boolean
  - title: class
    default: ""
    validation:
      is_a: text
---

<img if={@set == "devicon"} src={"https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/" | append(@name) | append("/") | append(@name) | append("-plain.svg")} data-d-component="Icon" data-variant={@variant} data-color={@color} data-size={@size} class={"d-icon " | append(@class)} />
<img if={@set == "lucide"} src={"https://unpkg.com/lucide-static@latest/icons/" | append(@name) | append(".svg")} data-d-component="Icon" data-variant={@variant} data-color={@color} data-size={@size} class={"d-icon" | append(@class)} />
