---
experimental:
  v2_templates: true
---

# Grid component

A `<Grid>` can be used to place elements in columns on your page. It's common to use `<Grid>` with either `<Card>` or `<Box>` components.

The component is responsive, which means the columns will collapse on smaller screens automatically.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Grid cols="3" gap="2">
        <Card>
          1
        </Card>
        <Card>
          2
        </Card>
        <Card>
          3
        </Card>
        <Card>
          4
        </Card>
      </Grid>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Grid component"
    <Grid cols="3" gap="2">
      <Card>
        ...
      </Card>

      ...
    </Grid>
    ```
  </Tab>
</Tabs>

## Attributes

The `<Grid>` component can be customized using the following attributes.

### Columns

You can set the number of columns with the `cols` attribute. Must be 1,2,3 or 4. Defaults to 1.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Grid cols="4" gap="2">
        <Card>
          1
        </Card>
        <Card>
          2
        </Card>
        <Card>
          3
        </Card>
        <Card>
          4
        </Card>
      </Grid>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">    
    ```html title="Grid component with 4 columns"
    // [!code word:cols:1]
    <Grid cols="4">
      <Card>
        ...
      </Card>

      ...
    </Grid>
    ```
  </Tab>
</Tabs>

### Gap

You can increase the gap between the grid items with the `gap` attribute. It must be a value between 0 and 5 inclusive.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Grid gap="5">
        <Card>
          1
        </Card>
        <Card>
          2
        </Card>
        <Card>
          3
        </Card>
        <Card>
          4
        </Card>
      </Grid>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```html title="Grid component with a large gap of 5"
    // [!code word:gap:1]
    <Grid gap="5">
      <Card>
        ...
      </Card>

      ...
    </Grid>
    ```
  </Tab>
</Tabs>
