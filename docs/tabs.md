# Tabs and subtabs

Tabs and subtabs can be used to split your documentation into logical sections. Each section will have its own navigation and its content lives in its own subdirectory.

![tabs and subtabs screenshot](/_assets/tabs-subtabs-dark.png)

## How tabs work

The basics of how tabs and subtabs work is as follows:

- Tabs show up at the top of your project
- Tabs may have subtabs, which will be shown above the main navigation
- Each tab or subtab has its own navigation structure
- If there is only one tab, no tabs are shown
- If a tab has only one subtab (like in this demo), the subtabs section isn't shown for that tab

## Defining tabs

Tabs are defined in your `docapella.yaml` file. Let's look at an example with both tabs and subtabs.

```yaml title="docapella.yaml · Tabs and subtabs"
tabs:
  - label: Home  # This is the default tab
    path: /
  - label: SDK   # This is another top-level tab
    path: /sdk
    subtabs:     # Subtabs under the SDK tab
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
  - label: API Reference  # A final top level tab
    path: /api/
```

Under the `tabs:` key, we define a list of 3 tabs, each with a label and path.

The second tab additionally has a `subtabs` field, which lists 3 subtabs.

## Navigation

Every tab and subtab gets its own `navigation.yaml` file - this allows you to have multiple tabs with different navigation structures.

Create a `navigation.yaml` in the subdirectory of the tab or subtab for Docapella to use.

For example, if we have a tab with the path `/sdk`, we can create a `sdk/navigation.yaml` file for it.

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

You can find the supported icon sets [here](/components/icon.md).
