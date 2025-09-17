---
attributes:
  - title: class
    required: false
    default: ""
    validation:
      is_a: text
  - title: src
    required: true
    validation:
      is_a: text
  - title: src_dark
    validation:
      is_a: text
  - title: alt
    validation:
      is_a: text
  - title: zoomable
    validation:
      is_a: boolean
    default: true
---

<img if={@src_dark && @src} src={@src} alt={@alt} class={"light-only " | append(@class)} data-zoomable={@zoomable} />
<img elseif={@src} src={@src} alt={@alt} class={@class} data-zoomable={@zoomable} />
<img if={@src_dark} src={@src_dark} alt={@alt} class={"dark-only " | append(@class)} data-zoomable={@zoomable} />
