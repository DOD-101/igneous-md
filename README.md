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
- Export to pdf
- Works offline

## To-do

- [ ] Write e2e tests (and benchmarks with hyperfine?)

- [ ] Create packages

- [ ] Allow multiple running instances at the same time on different docs

- [ ] Add change streaming API

    - [ ] Editor integration via plugin (Neovim)

- [x] Fix bug relating to indented GFM notes

- [x] Add github theme closer to github itself (limit width and center content)

- [x] Optimize performance

    - [x] Fix slow shutdown times

## Installation

### NixOS

If your system uses nix [flakes](https://nix.dev/concepts/flakes.html) you can do the following:

```nix
# flake.nix
inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    igneous-md = {
      url = "github:DOD-101/igneous-md"; # or /ref/tags/0.3.0 to pin a version
      inputs.nixpkgs.follows = "nixpkgs";
    };
}
```

and then

```nix
# configuration.nix
{
    inputs,
    ...
}:
{
    environment.systemPackages = [
        inputs.igneous-md.packages."${pkgs.stdenv.hostPlatform.system}".igneous-md-release
    ];
}
```

> [!IMPORTANT]
> This will build the project from source

### Via Cargo

`cargo install igneous-md`

> [!IMPORTANT]
> This will build the project from source (see build dependencies)

You must also install `webkit-gtk 2.3x+`.

### Building from source

1. Clone the repo `git clone https://github.com/DOD-101/igneous-md.git`

2. Install the following build dependencies:

    - rust 1.89+
    - gtk4 (also a runtime dependency)
    - webkit-gtk 2.3x+ (also a runtime dependency)
    - esbuild

    #### Arch Linux

    ```sh
    sudo pacman -Syu gtk4 webkitgtk-6.0 base-devel
    ```

    #### Ubuntu

    ```bash
    sudo apt update && apt install libgtk-4-dev libwebkitgtk-6.0-dev build-essential libssl-dev
    ```

> [!NOTE]
> Feel free to add your distro here! PRs welcome!

3. Run `cargo build --release`

4. Finally run `cargo install --path crates/igneous-md`

## Usage

```
igneous-md view path/to/file.md
```

## FAQ

> Or at least questions I think people could ask

1. How do I view my markdown in the browser?

   Simply pass the `--browser` flag. For all options run `igneous-md --help`

   <!-- TODO: Add full --help auto-generated output -->

2. How can I change the order of color schemes?

   Prefix the css file names with numbers e.g:

   ```
   00_github-dark.css
   01_github-light.css
   ```

## Configuration

To get started run `igneous-md generate-config` (will run by automatically if you view a file without the config dir `~/.config/igneous-md/`)

> [!NOTE]
> Requires `curl` to be installed.

### Layout

```sh
# in ~/.config/igneous-md/

css # general styling
├── github-markdown-dark.css
├── github-markdown-light.css
└── hljs # codeblocks
    ├── github-dark.css
    └── github-light.css
```

## Keybindings

| Key    | Description                  |
| ------ | ---------------------------- |
| `c`    | Go to next color scheme      |
| `C`    | Go to previous color scheme  |
| `e`    | Export html                  |
| `hjkl` | Vim bindings for moving      |

## Neovim Integration

This allows to to launch igneous-md directly from your editor on the current buffer

```lua
local job_id = -1
vim.keymap.set("n", "gm", function()
	if job_id ~= -1 then
		vim.fn.jobstop(job_id)
	end
	local current_buffer_path = vim.fn.expand("%")
	job_id = vim.fn.jobstart({ "igneous-md", "view", current_buffer_path })
end, {})
```

## Converting md to html

`igneous-md convert <PATH>`

## A markdown viewer Framework?

Yes. It's simpler than it sounds.

igneous-md works by using a server in the background and then communicating viewers
via a websocket connection, so anyone can create their own viewer by implementing this websocket-based json protocol.

The benefits of this being you receive all of the hot-reloading and conversion from md to html for free,
while having full freedom to implement the viewer frontend however you you like.

The only real limitation on this is what the websocket json-protocol is written to support. (PRs welcome)

This aspect of igneous-md is still experimental, but if you want to get started check out [`./crates/igneous-md/src/ws/msg.rs`](./crates/igneous-md/src/ws/msg.rs)
and have a look at what is possible right now and what messages you need to handle from the server.

If you want to only use igneous-md for this be sure to disable the `viewer` cargo feature.

### Writing your own viewer

1. Implement the client-side (viewer) code for handling communication with the server. See: [`./crates/igneous-md/src/ws/msg.rs`](./crates/igneous-md/src/ws/msg.rs)

2. Assets (currently just images) are loaded via a custom URI scheme `asset://`. This means to facilitate the loading of images the client needs to handle this URI scheme. (This might change in the future to move this responsibility over to the server-side)

*That's it.*

For an example look at the implementation of `igneous-md-viewer` in `./crates/igneous-md-viewer/`.


## Attribution

Many thanks to all the people, who have created/contributed to technology used in the creation this project.

GitHub for their markdown styling and markdown-alert [icons](./assets).

## License

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT License](https://opensource.org/license/MIT)

at your option.
