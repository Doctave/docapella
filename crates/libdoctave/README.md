libdoctave | A documentation processing system 
==============================================

libdoctave is the library that is responsible for:

* Building and structuring a documentation site
* Building various navigation structures
* Rendering Markdown in our specific flavors
* Rendering the whole documentation site in various different configurations

It is used in

* The Doctave backend, `rumba`
* The Desktop app
* The browser frontend, via WASM

## Tasks

* [ ] Rendering Markdown
  * [ ] Basic Markdown syntax
  * [ ] Expose AST as Rust/JSON structures
  * [ ] Extensions
    * [ ] Mermaid JS
    * [ ] KaTeX
* [ ] Navigation
  * [ ] Build automatic navigation from file hierarchy
  * [ ] Explicit navigation
  * [ ] Allow overriding base url path
* [ ] Persisting
  * [ ] Save changes to a document back to disk