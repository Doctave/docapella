# Header and footer

Docapella supports customizing the header and footer of your documentation.

## Header

The header is the top bar that contains your logo, links, and call-to-action button.

```yaml title="docapella.yaml · Header"
header:
  links:
    - label: GitHub
      external: https://github.com/example/example
  cta:
    label: Sign Up
    external: https://example.com
```

### Links

The links in the header are shown in the order you specify them. You can specify links to external URLs, or to internal pages.

To link to an internal page, you can use the `href` property. This will link to a Markdown page in your project. For external pages use the `external` property.

```yaml title="docapella.yaml · Header · Links"
header:
  links:
    - label: GitHub
      external: https://github.com/example/example
    - label: Home
      href: /
```

### Call-to-action button

The call-to-action button is a button that links to an external URL. It is shown in the header.

```yaml title="docapella.yaml · Header · Call-to-action button"
header:
  links:
    - label: GitHub
      external: https://github.com/example/example
  # A more prominent call-to-action button
  cta:
    label: Sign Up
    external: https://example.com
```

## Footer

The footer is the bottom bar that contains your links and social media profiles.

```yaml title="docapella.yaml · Footer"
footer:
  links:
    - label: GitHub
      external: https://github.com/Doctave/docapella
    - label: Home
      href: /
```

### Links

The links in the footer are shown in the order you specify them. You can specify links to external URLs, or to internal pages.

To link to an internal page, you can use the `href` property. This will link to a Markdown page in your project.

```yaml title="docapella.yaml · Footer · Links"
footer:
  links:
    - label: GitHub
      external: https://github.com/Doctave/docapella
    - label: Home
      href: /
```

### Social media profiles

You can add social media profiles to your footer. This will add a link to the social media profile, and a small icon next to it.

```yaml title="docapella.yaml · Footer · Social media profiles"
footer:
  links:
    - label: Support
      external: https://example.com/support
    - label: Dashboard
      href: https://example.com/dashboard
  github: https://github.com/example
  twitter: https://twitter.com/example
  linkedin: https://www.linkedin.com/company/example
```
