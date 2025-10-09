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

<Fragment if={(@set == "devicon") && (@variant == "boxed")}>
  <div data-d-component="IconBox" data-color={@color} data-size={@size} class={@class}>
    <div role="img" aria-label={@name | append(" icon")} data-d-component="Icon" data-color={@color} style={"mask-image: url(https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/" | append(@name) | append("/") | append(@name) | append("-plain.svg); -webkit-mask-image: url(https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/") | append(@name) | append("/") | append(@name) | append("-plain.svg);")}></div>
  </div>
</Fragment>

<div if={(@set == "devicon") && (@variant != "boxed")} role="img" aria-label={@name | append(" icon")} data-d-component="Icon" data-color={@color} data-size={@size} class={"d-icon " | append(@class)} style={"mask-image: url(https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/" | append(@name) | append("/") | append(@name) | append("-plain.svg); -webkit-mask-image: url(https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/") | append(@name) | append("/") | append(@name) | append("-plain.svg);")}></div>

<Fragment if={(@set == "lucide") && (@variant == "boxed")}>
  <div data-d-component="IconBox" data-color={@color} data-size={@size} class={@class}>
    <div role="img" aria-label={@name | append(" icon")} data-d-component="Icon" data-color={@color} style={"mask-image: url(https://unpkg.com/lucide-static@latest/icons/" | append(@name) | append(".svg); -webkit-mask-image: url(https://unpkg.com/lucide-static@latest/icons/") | append(@name) | append(".svg);")}></div>
  </div>
</Fragment>

<div if={(@set == "lucide") && (@variant != "boxed")} role="img" aria-label={@name | append(" icon")} data-d-component="Icon" data-color={@color} data-size={@size} class={"d-icon " | append(@class)} style={"mask-image: url(https://unpkg.com/lucide-static@latest/icons/" | append(@name) | append(".svg); -webkit-mask-image: url(https://unpkg.com/lucide-static@latest/icons/") | append(@name) | append(".svg);")}></div>
