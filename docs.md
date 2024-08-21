# Docs 

> While Igneous-md is still in early development, it's already mostly functional.

## Installing 

The simplest way to install is to run `cargo install igneous-md`

## Getting started

1. Make sure you have `webkit2gtk` installed on your system. You also need to install the `segoe-ui`-font and `apple-emoji`s for the GitHub style. 

2. Create the config directory at `~/.config/igneous-md/`
<!-- TODO: Add example config.toml -->
3. In the config directory add `config.toml` (optional) & `css/`
<!-- TODO: Add example css -->
4. Add some css files in the `css/` directory. These are the files that will be used to style your markdown. 

5. Find a markdown file you want to view and run `igneous-md --path path/to/file.md`


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

    Simply pass the `--browser` flag. For all options run `igneous-md`

