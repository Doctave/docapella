# Tabs Component

The Tabs component enables you to organize content into multiple tabs, allowing users to switch between different sections of information or functionality within a single interface.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Box pad="2">
        <Tabs>
          <Tab title="First Tab">
            First tab content
          </Tab>
          <Tab title="Second Tab">
            Second tab content
          </Tab>
          <Tab title="Third Tab">
            Third tab content
          </Tab>
        </Tabs>
      </Box>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Tabs component structure"
    <Tabs>
        <Tab title="...">
            ...
        </Tab>
        <Tab title="...">
            ...
        </Tab>
    </Tabs>
    ```
  </Tab>
</Tabs>

## Structure

The `<Tabs>` (plural) component expects as children a sequence of `<Tab>` (singular) components.

Passing anything other than a `<Tab>` components as a child to the `<Tabs>` component will result in an error.

## Attributes

The `<Tabs>` and `<Tab>` components can be customized using the following attributes.

### Tab title

Each `<Tab>` (singular) component must have a `title` attribute, which specifies the title of the tab.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Box pad="2">
        <Tabs>
          <Tab title="First Tab">
            First tab content
          </Tab>
          <Tab title="Second Tab">
            Second tab content
          </Tab>
          <Tab title="Third Tab">
            Third tab content
          </Tab>
        </Tabs>
      </Box>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Tabs component structure"
    <Tabs>
        <Tab title="First Tab">
            ...
        </Tab>
        <Tab title="Second Tab">
            ...
        </Tab>
        <Tab title="Third Tab">
            ...
        </Tab>
    </Tabs>
    ```
  </Tab>
</Tabs>

