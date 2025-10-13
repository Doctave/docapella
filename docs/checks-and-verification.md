# Checks and verification

Docapella checks your project for common errors and issues, such as broken links and syntax errors.

There is nothing to configure, and it will run automatically when you run `docapella build`, or when you make changes to your Markdown files during `docapella dev`.

## Links

Internal links are checked for validity. If a link is broken, you will get a warning in the console.

External links are currently not checked for validity at this time.

## Syntax

Docapella checks your Markdown files for syntax errors. Unlike traditional Markdown flavors, Docapella uses a custom syntax that is more strict for its component system.

For exmaple, if you have a mismatched close tag, you will get an error:

```plain title="Error message for a component with mismatched tags"
Unexpected closing tag `</Box>`, expected corresponding closing tag for `<Card>`

    1 │ <Card> Content </Box>
        ▲              ▲
        │              ╵
        └─ Opening tag
                       ╷
                       └─ Expected close tag
```
