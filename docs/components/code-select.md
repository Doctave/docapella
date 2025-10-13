# CodeSelect Component

The CodeSelect component allows to conveniently present multi-language code examples within your documentation.

Wrap one or more normal Markdown code blocks within `<CodeSelect>...</CodeSelect>` to turn it into a CodeSelect component.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <CodeSelect title="Capitalize">
      ```typescript
      function capitalize(str: string): string {
        return str.charAt(0).toUpperCase() + str.slice(1);
      }
      ```
      ```rust
      fn capitalize(str: String) -> String {
        str.chars().next().unwrap().to_uppercase().collect::<String>() + &str[1..]
      }
      ```
      </CodeSelect>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ````html title="CodeSelect component"
    <CodeSelect title="Capitalize">
    ```typescript
    function capitalize(str: string): string {
      return str.charAt(0).toUpperCase() + str.slice(1);
    }
    ```
    ```rust
    fn capitalize(str: String) -> String {
      str.chars().next().unwrap().to_uppercase().collect::<String>() + &str[1..]
    }
    ```
    </CodeSelect>
    ````
  </Tab>
</Tabs>

## Attributes

The `<CodeSelect>` component can be customized using the following attributes and configuration.

### Title

The `<CodeSelect>` shows a title describing the code blocks it contains.

It's set using the `<CodeSelect title="...">` attribute. This is a **required** attribute.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <CodeSelect title="Descriptive explanation">
      ```typescript
      function capitalize(str: string): string {
        return str.charAt(0).toUpperCase() + str.slice(1);
      }
      ```
      ```rust
      fn capitalize(str: String) -> String {
        str.chars().next().unwrap().to_uppercase().collect::<String>() + &str[1..]
      }
      ```
      </CodeSelect>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ````html title="CodeSelect component"
    // [!code word:title:1]
    <CodeSelect title="Descriptive explanation">
      ...
    </CodeSelect>
    ````
  </Tab>
</Tabs>

<br />

#### Overriding the title per code block

You can overwrite the title of individual code blocks by specifying a title on the code block.
When selecting a code block, the title is updated to reflect the selected code block.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <CodeSelect title="Capitalize">
      ```typescript title="Capitalize in TypeScript"
      function capitalize(str: string): string {
        return str.charAt(0).toUpperCase() + str.slice(1);
      }
      ```
      ```rust title="Capitalize in Rust"
      fn capitalize(str: String) -> String {
        str.chars().next().unwrap().to_uppercase().collect::<String>() + &str[1..]
      }
      ```
      </CodeSelect>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ````html title="CodeSelect component"
    <CodeSelect title="Descriptive explanation">
      ```typescript title="Capitalize in TypeScript"

      ```
      ```rust title="Capitalize in Rust"

      ...
      ```
    </CodeSelect>
    ````
  </Tab>
</Tabs>

### Dropdown label

You can overwrite each code block's label, which is displayed in the dropdown menu, by specifying a `label` attribute.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      <CodeSelect title="Capitalize">
        ```typescript label="Custom language label"
        function capitalize(str: string): string {
          return str.charAt(0).toUpperCase() + str.slice(1);
        }
        ```
        ```rust
        fn capitalize(str: String) -> String {
          str.chars().next().unwrap().to_uppercase().collect::<String>() + &str[1..]
        }
        ```
      </CodeSelect>
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
      ````jsx
      <CodeSelect title="Capitalize">
      ```typescript label="Custom language label"

      ```
      </CodeSelect>
      ````
  </Tab>
</Tabs>

