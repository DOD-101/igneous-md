# 🌋 Igneous-md

![](<https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FDOD-101%2Figneous-md%2Frefs%2Fheads%2Fmaster%2Fcrates%2Figneous-md%2FCargo.toml&query=package.version&label=Version&color=rgb(20%2C20%2C20)>)
[![](https://img.shields.io/badge/Crates.io-orange?style=flat&link=https%3A%2F%2Fcrates.io%2Fcrates%2Figneous-md)](https://crates.io/crates/igneous-md)

> The simple and lightweight markdown framework / viewer written in rust

Igneous-md is a [gfm](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) compatible markdown viewer and viewer framework with a focus of staying lightweight and extensible

## Features

- Syntax highlighting similar to GitHub using [highlight.js](https://github.com/highlightjs/highlight.js)
- Standalone markdown viewer outside browser
- Switching of stylesheets on the fly
- Ability to add custom CSS
- Export generated HTML
- Works offline

## To-do

- [ ] Optimize performance

    - [ ] Fix slow shutdown times

- [ ] Add github theme closer to github itself (limit width and center content)

    - Either as css or with a keybind

- [ ] Write tests with hyperfine

- [ ] Create packages

- [ ] Allow multiple running instances at the same time on different docs

## Usage

```
igneous-md view path/to/file.md
```

For more information see [docs.md](./docs.md)

## A markdown viewer Framework?

Yes. It's simpler than it sounds.

Since igneous-md works by using a server in the background and then communicates with the built-in viewer, as well as the browser,
using https and websockets anyone could use this to write their own viewer.

The benefits of this being you would receive all of the hot-reloading and conversion from md to html for free,
while having full freedom to implement your viewer however they would like.

The only real limitation on this is what the websocket json-protocol is written to support. (PRs welcome)

This aspect of igneous-md is still experimental, but if you already want to get started check out [`./crates/igneous-md/src/handlers/ws.rs`](./crates/igneous-md/src/handlers/ws.rs)
and have a look at what is possible right now.

If you want to only use igneous-md for this be sure to disable the `viewer` cargo feature.

## Attribution

Many thanks to all the people, who have created/contributed to technology used in the creation this project.

GitHub for their markdown styling and markdown-alert [icons](./assets).

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT License](https://opensource.org/license/MIT)

at your option.
