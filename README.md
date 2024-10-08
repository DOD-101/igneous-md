# 🌋 Igneous-md

![](<https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FDOD-101%2Figneous-md%2Fmaster%2FCargo.toml&query=package.version&label=Version&color=rgb(20%2C20%2C20)>)
[![](https://img.shields.io/badge/Crates.io-orange?style=flat&link=https%3A%2F%2Fcrates.io%2Fcrates%2Figneous-md)](https://crates.io/crates/igneous-md)

> The simple and lightweight markdown viewer written in rust

Igneous-md is a [gfm](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) compatible markdown viewer with a focus of staying lightweight and minimal.

## Features

- Syntax highlighting similar to GitHub using [highlight.js](https://github.com/highlightjs/highlight.js)
- Standalone markdown viewer outside browser
- Switching of stylesheets on the fly
- Ability to add custom CSS
- Export generated HTML
- Works offline

## To-do

- [ ] Fix bugs

- [ ] Create packages

- [ ] Write tests

- [x] Add default css / examples

- [x] Add syntax highlighting

- [x] Move from `fetch` to websockets

- [x] Export HTML

- [x] Add docs

## Usage

```
igneous-md --path path/to/file.md
```

For more information see [docs.md](./docs.md)

## How does it work?

Igneous-md works by running a lightweight server in the background, to which any number of clients may connect to. This means you can view your markdown in the provided viewer, or if you prefer in the browser.

## Attribution

Many thanks to all the people, who have created/contributed to technology used in the creation this project.

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT License](https://opensource.org/license/MIT)

at your option.
