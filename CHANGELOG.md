# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CLI argument support using `clap` (path, verbose, help, version).
- Modularized UI rendering engine for better maintainability.
- Side-by-side diff visualization for git repositories.
- In-TUI settings editor for JSON configuration.
- Theme selector with Nord, Catppuccin, Dracula and Monochrome themes.
- GitHub Actions CI workflow for automated testing and linting.
- Security policies and contributor guidelines.

### Changed
- Refactored monolithic `render.rs` into focused modules.

### Fixed
- Improved large file handling and diff truncation.
