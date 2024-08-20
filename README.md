# 🌋 Igneous-md 

> The simple and lightweight markdown viewer written in rust

Igneous-md is a [gfm](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) compatible markdown viewer with a focus of staying lightweight and minimal. 

## Features 

- Standalone markdown viewer outside browser
- Switching of stylesheets on the fly
- Ability to add custom CSS
- Works offline

## To-do

- [ ] Add docs

- [ ] Fix bugs

- [ ] Move from `fetch` to websockets 

- [ ] Create packages

- [ ] Export HTML

- [ ] Write tests

- [ ] Add syntax highlighting

## How to use

```
igneous-md --path path/to/file.md
```

For more information see [docs.md](./docs.md)

## How does it work?

Igneous-md works by running a lightweight server in the background, to which any number of clients may connect to. This means you can view your markdown in the provided viewer, or if you prefer in the browser. 

## Attribution

Special thanks to all the people, who have created the assets used in creating this project.

