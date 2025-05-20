---
attributes:
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
      is_one_of:
        - "_self"
        - "_blank"
  - title: variant
    default: "primary"
    validation:
      is_one_of:
        - "primary"
        - "secondary"
        - "ghost"
        - "outline"
  - title: size
    default: "md"
    validation:
      is_one_of:
        - "sm"
        - "md"
        - "lg"
  - title: width
    default: "fit-content"
    validation:
      is_one_of:
        - "full"
        - "fit-content"
---


<a if={@download_as} download={@download_as} class="d-button" href={@href} data-variant={@variant} data-size={@size} data-width={@width} target="_blank">
  <Slot />
</a>
<a elseif={@download} download="" class="d-button" href={@href} data-variant={@variant} data-size={@size} data-width={@width} target="_blank">
  <Slot />
</a>
<a else class="d-button" target={@target} href={@href} data-variant={@variant} data-size={@size} data-width={@width}>
  <Slot />
</a>
