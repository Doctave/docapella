# Components

Docapella comes with a library of **UI and layout components**. You've already seen some in action on the [`README.md`](/) page. Here are some other examples of what you can do with Docapella components.


## Multi-language code examples

Very useful when you have code snippets in multiple languages.

**Syntax:**
~~~jsx title="README.md"
<CodeSelect title="Hello world in multiple language">
  ```typescript
  console.log("Hello, world")
  ```

  ```rust
  println!("Hello, world")
  ```
</CodeSelect>
~~~

**Output:**

<CodeSelect title="Hello world in multiple language">
  ```typescript
  console.log("Hello, world")
  ```

  ```rust
  println!("Hello, world")
  ```
</CodeSelect>

## Tabs

**Syntax:**
~~~jsx title="README.md"
<Tabs>
    <Tab title="First">
        This is the content for tab **1**.
    </Tab>
    <Tab title="Second">
        ```
        This is the content for tab 2 inside a code block
        ```
    </Tab>
    <Tab title="Third">
        This is the content for tab **3**.
    </Tab>
</Tabs>
~~~

**Output:**

<Tabs>
    <Tab title="First">
        This is the content for tab **1**.
    </Tab>
    <Tab title="Second">
        ```
        This is the content for tab 2 inside a code block
        ```
    </Tab>
    <Tab title="Third">
        This is the content for tab **3**.
    </Tab>
</Tabs>

## Steps

`<Steps>` let you describe a sequence of steps required for an action.

**Syntax:**
```jsx title="README.md"
<Steps>
  <Step title="Step one">
    Requirements for **step one**
  </Step>
  <Step title="Step two">
    Requirements for **step two**
  </Step>
</Steps>
```

**Output:**

<Steps>
  <Step title="Step one">
    Requirements for **step one**
  </Step>
  <Step title="Step two">
    Requirements for **step two**
  </Step>
</Steps>
