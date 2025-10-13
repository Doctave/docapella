# Code Component

The Code component allows you to display code snippets within your application or documentation. It supports specifying the language and an optional title for better organization and clarity.

While the syntax is the same is with regular Markdown code blocks, Doctave supports additional features no available in most Markdown implementation.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      ```javascript title="server.ts"
      const http = require('http');

      const hostname = '127.0.0.1';
      const port = 3000;

      const server = http.createServer((req, res) => {
        res.statusCode = 200;
        res.setHeader('Content-Type', 'text/plain');
        res.end('Hello World\n');
      });

      server.listen(port, hostname, () => {
        console.log(`Server running at http://${hostname}:${port}/`);
      });
      ```
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ````markdown title="Code component"
    ```javascript title="server.ts"
    const http = require('http');

    const hostname = '127.0.0.1';
    const port = 3000;

    const server = http.createServer((req, res) => {
      res.statusCode = 200;
      res.setHeader('Content-Type', 'text/plain');
      res.end('Hello World\n');
    });

    server.listen(port, hostname, () => {
      console.log(`Server running at http://${hostname}:${port}/`);
    });
    ```
    ````
  </Tab>
</Tabs>

## Attributes

The `<Code>` component can be customized using the following attributes and configuration.

### Language and syntax highlighting

To activate syntax highlighting, add a language name after the three backticks (```).

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      ```elixir
      defmodule MyApp.Server do
        def start(_type, _args) do
          IO.puts "Starting MyApp.Server"
        end
      end
      ```
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ````markdown title="Code block with Elixir syntax highlighting"
    ```elixir
    defmodule MyApp.Server do
      def start(_type, _args) do
        IO.puts "Starting MyApp.Server"
      end
    end
    ```
    ````
  </Tab>
</Tabs>

### Title

You may also specify a title for the code block, which is displayed above the code.

You **must** specify a language for the title to be displayed. If you don't want syntax highlighting for the code block, you can use the `plaintext` language.

<Tabs>
  <Tab title="Preview">
    <Component.ComponentDemo>
      ```typescript title="Uploading a file"
      async function uploadFile(file: File, url: string): Promise<Response> {
        const formData = new FormData();
        formData.append('file', file);
        return fetch(url, { method: 'POST', body: formData });
      }
      ```
    </Component.ComponentDemo>
  </Tab>
  <Tab title="Code">
    ````markdown title="Code block with a title"
    ```typescript title="Uploading a file"
    async function uploadFile(file: File, url: string): Promise<Response> {
      const formData = new FormData();
      formData.append('file', file);
      return fetch(url, { method: 'POST', body: formData });
    }
    ```
    ````
  </Tab>
</Tabs>


