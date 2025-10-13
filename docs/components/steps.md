# Steps Component

The `<Steps>` component allows you to guide users through a series of sequential steps, providing instructions and content for each step along the way.

Each step can have arbitrary Markdown or other components.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Box pad="2">
        <Steps>
          <Step title="Step 1">
            Explanation of step 1
          </Step>
          <Step title="Step 2">
            Explanation of step 2
          </Step>
          <Step title="Step 3">
            Explanation of step 3
          </Step>
        </Steps>
      </Box>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Step component structure"
    <Steps>
      <Step title="...">
        ...
      </Step>
      <Step title="...">
        ...
      </Step>
    </Steps>
    ```
  </Tab>
</Tabs>

## Structure

The `<Steps>` (plural) component expects as children a sequence of `<Step>` (singular) components.

```jsx title="Step component structure"
<Steps>
  <Step title="...">
    ...
  </Step>
  <Step title="...">
    ...
  </Step>
</Steps>
```

Passing anything other than a `<Step>` components as a child to the `<Steps>` component will result in an error.


## Attributes

The `<Steps>` and `<Step>` components can be customized using the following attributes.

### Step title

Each `<Step>` (singular) component must have a `title` attribute, which specifies the title of the step.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <Box pad="2">
        <Steps>
          <Step title="First step">
            Procure volcanic island lair
          </Step>
          <Step title="Second step">
            ???
          </Step>
          <Step title="Third step">
            Profit!
          </Step>
        </Steps>
      </Box>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ```jsx title="Step component structure"
    <Steps>
      <Step title="...">
        ...
      </Step>
      <Step title="...">
        ...
      </Step>
    </Steps>
    ```
  </Tab>
</Tabs>

