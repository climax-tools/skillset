# Skillset

A CLI package manager for AI agent skills. Install, manage, and organize skills across multiple agent frameworks with simple npm-like commands.

## Quick Start

### Installation
```bash
cargo install skillset
```

### Basic Usage
```bash
# Install a skill
skillset add file-analyzer@1.0.0

# Install from a specific source
skillset add my-skill --source git:https://github.com/user/repo

# List installed skills
skillset list

# Remove a skill
skillset remove file-analyzer
```

### Configuration
Create `skillset.json` in your project:

```json
{
  "skills": {
    "file-analyzer": "1.0.0",
    "@user/web-scraper": "2.1.0",
    "custom-skill": {
      "version": "3.0.0",
      "source": "git:https://github.com/custom/repo",
      "convention": "autogpt"
    }
  },
  "registry": "ghcr.io/skillset",
  "conventions": ["autogpt", "langchain"]
}
```

## Features

- **Multi-Framework Support**: Works with Auto-GPT, LangChain, and custom agent frameworks
- **Smart Organization**: Automatically organizes skills by framework conventions
- **Multiple Sources**: Install from Git repositories, OCI registries, or local paths
- **Version Management**: Pin specific versions or use `latest`
- **Scoped Namespaces**: Use `@user/skill` format for community skills
- **Zero-Configuration Caching**: Automatic cross-project skill sharing

## CLI Reference

### Skill Management
```bash
skillset add <skill>[@<version>] [--source <source>] [--convention <convention>]
skillset remove <skill>
skillset list [--verbose]
skillset update [skill]
skillset info <skill>
```

### Conventions
```bash
skillset convention list
skillset convention enable <name>
skillset convention disable <name>
```

### Publishing
```bash
skillset publish <path> <reference>
```

## Supported Sources

- **Git**: `git:https://github.com/user/repo` or direct GitHub URLs
- **OCI**: `oci:ghcr.io/user/skill:v1.0.0` (default for simple names)
- **Local**: `./local-skill` or absolute paths

## Framework Support

- **Auto-GPT**: Automatically detected and organized as `skills/autogpt/{name}/`
- **LangChain**: Automatically detected and organized as `skills/langchain/{name}/`
- **Custom**: User-defined conventions for any framework

## Reference Resolution

Simple skill names automatically resolve to OCI registries:

- `file-analyzer` → `oci:ghcr.io/skillset/file-analyzer:v1.0.0`
- `@user/skill` → `oci:ghcr.io/user/skill:v1.0.0`

## Project Structure

Skills are organized by framework:

```
project/
├── skillset.json
├── skills/
│   ├── autogpt/
│   │   └── file-analyzer/
│   └── langchain/
│       └── llm-tool/
└── .skillset/
    └── cache/
```

## Development

For architecture details and integration guidelines, see [AGENTS.md](./AGENTS.md).