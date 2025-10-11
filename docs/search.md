# Search

Docapella comes with a built-in search feature that lets you search your documentation. There is no setup required, and it works out of the box.

The search index is generated from your Markdown files and OpenAPI operations, and is updated automatically when you make changes to your documentation.

## Using the search

To use the search, you can press <kbd>Ctrl+K</kbd> on your keyboard, or click the magnifying glass icon in the top right corner of the header.

## Under the hood

The search index is powered by [ElasticLunr.js](https://elasticlunr.com/), which is a JavaScript-based full-text search library.

At build time, Docapella generated a `search.json` file that contains the search index. When the user searches, the index will be loaded into the browser for elasticlunr.

The index can be quite large for large projects (~megabytes), but the benefit of this approach is that there is zero infrastucture to manage.

