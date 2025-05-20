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

<i data-doctave-component="Icon" if={@set == "devicon"} data-variant={@variant} data-color={@color} data-size={@size} class={"devicon-" | append(@name) | append("-plain ") | append(@class)}></i>
<i data-doctave-component="Icon" if={@set == "lucide"}  data-variant={@variant} data-color={@color} data-size={@size} class={"icon-" | append(@name) | append(" ") | append(@class)}></i>
