# Tabs and subtabs

Tabs and subtabs can be used to split your documentation into logical sections. Each section will have its own navigation and its content lives in its own subdirectory.

![tabs and subtabs screenshot](/_assets/tabs-subtabs-dark.png)

This demo project has 2 tabs:

- A ["Home"](/) section that acts as your landing page
- An ["API Reference"](/api) section for documentation generated from an OpenAPI specification

## How tabs work

The basics of how tabs and subtabs work is as follows:

- Tabs show up at the top of your project
- Tabs may have subtabs, which will be shown above the main navigation
- Each tab or subtab has its own navigation structure
- If there is only one tab, no tabs are shown
- If a tab has only one subtab (like in this demo), the subtabs section isn't shown for that tab

## Defining tabs

Tabs are defined in your `docapella.yaml` file. Let's look at an example with both tabs and subtabs.

```yaml
tabs:
  - label: Home
    path: /
  - label: SDK
    path: /sdk
    subtabs:
    - path: /sdk
      label: Nebularis SDKs
      icon:
        set: lucide
        name: package
    - path: /sdk/python
      label: Python SDK
      icon:
        set: devicon
        name: python
    - path: /sdk/typescript
      label: Typescript SDK
      icon:
        set: devicon
        name: typescript
  - label: API Reference
    path: /api/
```

Under the `tabs:` key, we define a list of 3 tabs, each with a label and path.

The second tab additionally has a `subtabs` field, which lists 3 subtabs.

## Icons

You can add icons to your tabs and subtabs:

```yaml title="Tab with an icon from the Lucide icon set"
tab:
  - path: /sdk
    label: Nebularis SDKs
    icon:
      set: lucide
      name: package
```
