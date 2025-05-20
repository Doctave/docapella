---
attributes:
  - title: class
    required: false
    default: ""
    validation:
      is_a: text
  - title: href
    required: true
    validation:
      is_a: text
  - title: download
    required: false
    default: false
    validation:
      is_a: boolean
  - title: download_as
    required: false
    validation:
      is_a: text
  - title: target
    required: false
    default: "_self"
    validation:
      is_a: text
      is_one_of:
        - "_self"
        - "_blank"
---

<a if={@download_as} download={@download_as} href={@href} class={@class}>
  <Slot />
</a>
<a elseif={@download} download="" href={@href} class={@class}>
  <Slot />
</a>
<a else href={@href} target={@target} class={@class}>
  <Slot />
</a>
