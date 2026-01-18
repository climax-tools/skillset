# Changelog

All notable changes to the skillset crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-17

### Added
- Initial release of skillset CLI package manager
- Support for multiple agent frameworks (Auto-GPT, LangChain, Agent Skills)
- Git source fetching with caching
- OCI registry support (framework)
- Configuration management with JSON
- Convention-based skill organization
- npm-like CLI interface
- React best practices skill as canonical example

### Features
- Multi-framework support with pluggable conventions
- User-wide skill caching with deduplication
- Automatic reference resolution
- Version management and scoped namespaces
- CLI commands: add, remove, list, update, info, publish
- Development-focused documentation and examples