# Changelog

## [Unreleased]

### Added

- Generate more user friendly default config files
- Add the viewer as its own crate and a feature in the server part

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

