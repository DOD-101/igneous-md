# Changelog

## [Unreleased]

### Added

- Ability to adjust update rate

## 0.3.0 - 2025-02-16

### Added

- Added a binary to igneous-md-viewer to be able to launch to separate from the server
- Add hot-reloading for the config
- Add support for [markdown highlight notes](https://github.com/orgs/community/discussions/16925)
- Vim bindings to the viewer
- CLI completions

### Fixed

- Comment out notice in hljs css files
- Fix link to github hljs css file
- Greatly improve shutdown time
- Some task-list items (checklists) not having the correct classes

### Changed

- Make client connections persistent between files. This improves performance and helps simplify the code
- Clients persist their websocket connection to the server between files

## 0.2.1 - 2025-01-03

### Added

- Generate more user friendly default config files
- Add the viewer as its own crate and add a feature in the server part

### Changed

- Adjust the CLI to make subcommands usage more "normal"

### Fixed

- Panic when attempting to get the previous css file

## 0.2.0 - 2024-12-30

### Added

- Cargo feature to toggle config generation (at compile time)

### Fixed

- Various minor bugs

### Changed

- Switch form [rouille](https://github.com/tomaka/rouille) to [rocket](https://rocket.rs/)
- Move all communication between client and server to Websockets
