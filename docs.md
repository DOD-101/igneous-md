# Docs

> While Igneous-md is still in development, it's already very functional.

## Installing

The simplest way to install is to run `cargo install igneous-md`

Or if you're using nix you can take advantage of the flake. Be sure to use `igneous-md-release`.

## Getting started

1. Make sure you have `webkit2gtk` (specifically ABI version `4.1`) installed on your system. <br>
It's also recommended to install the `noto-fonts-color-emoji` font (or whatever it is called in your package manager of choice). <br>
*(not needed with the flake)*

2. Find a markdown file you want to view and run `igneous-md view path/to/file.md`

## Configuration

Configuring igneous-md is super simple (assuming you know some basic CSS).

Simply copy one of the given CSS files and change whatever you want. If you want to change the highlighting of the code blocks, have a look in `hljs/`.

## Default Config

The default config contains the css files for the github theme. (light & dark)

You can generate the default config by either running `igneous-md generate-config` or launching igneous-md without having `.config/igneous-md/css`.

The default css files also provide a great starting point for creating your own themes.

Note: Config generation is only available if compiled with `--features generate_config` (it is a default feature).

## Keybindings

| Key    | Description                  |
| ------ | ---------------------------- |
| `c`    | Go to next color scheme      |
| `C`    | Go to previous color scheme  |
| `e`    | Export html                  |
| `hjkl` | Vim bindings                 |

## Integration with Neovim

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

## Converting html to md

Igneous-md allows you to quickly convert your `.md` files to html:

`igneous-md convert <PATH>`

## FAQ

> Or at least questions I think people could ask

1. How do I view my markdown in the browser?

   Simply pass the `--browser` flag. For all options run `igneous-md --help`

2. How can I change the order of color schemes?

   Prefix the css file names with numbers e.g:

   ```
   00_github-dark.css
   01_github-light.css
   ```
