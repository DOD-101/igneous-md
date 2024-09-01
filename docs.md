# Docs

> While Igneous-md is still in early development, it's already mostly functional.

## Installing

The simplest way to install is to run `cargo install igneous-md`

## Getting started

1. Make sure you have `webkit2gtk` installed on your system. You also need to install the `segoe-ui`-font and `apple-emoji`s for the GitHub style.

2. Create the config directory at `~/.config/igneous-md/`

3. Copy the contents of `example/` to your config directory

4. Find a markdown file you want to view and run `igneous-md --path path/to/file.md`

## Configuration

Configuring igneous-md is super simple (assuming you know some basic CSS).

Simply copy one of the given CSS files and change whatever you want. If you want to change the highlighting of the code blocks, have a look in `hljs/`.

## Keybindings

| Key | Description                  |
| --- | ---------------------------- |
| `c` | Go to next color scheme      |
| `C` | Go to previous color scheme  |
| `e` | Export html                  |
| `E` | Export html ( `<body>` only) |

## Integration with Neovim

```lua
local job_id = -1
vim.keymap.set("n", "gm", function()
	if job_id ~= -1 then
		vim.fn.jobstop(job_id)
	end
	local current_buffer_path = vim.fn.expand("%")
	job_id = vim.fn.jobstart({ "igneous-md", "--path", current_buffer_path })
end, {})
```

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
