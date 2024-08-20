# Docs 

> While Igneous-md is still in early development, it's already mostly functional.

## Getting started

1. Make sure you have  and `webkit2gtk` installed on your system.

2. Create the config directory at `~/.config/igneous-md/`
<!-- TODO: Add example config.toml -->
3. In the config directory add `config.toml` (optional) & `css/`
<!-- TODO: Add example css -->
4. Add some css files in the `css/` directory. These are the files that will be used to style your markdown. 

5. Find a markdown file you want to view and run `igneous-md --path path/to/file.md`


## Integration with Neovim

This will open igneous-md every time you open a `.md` file.

```lua
local Job_ID = -1
vim.api.nvim_create_autocmd({ "BufEnter", "BufRead", "BufNewFile" }, {
	pattern = "*.md",
	once = false,
	callback = function()
		if Job_ID ~= -1 then
			vim.fn.jobstop(Job_ID)
		end
		local current_buffer_path = vim.fn.expand("%")
		-- print(current_buffer_path)
		Job_ID = vim.fn.jobstart({ "igneous-md", "--path", current_buffer_path })
		-- print(Job_ID)
	end,
})
```

## FAQ

> Or at least questions I think people could ask

1. How do I view my markdown in the browser?

    Simply pass the `--browser` flag. For all options run `igneous-md`

