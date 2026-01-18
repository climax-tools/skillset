# Skillset

A CLI package manager for AI agent skills. Install, manage, and organize skills across multiple agent frameworks with simple npm-like commands.

## Quick Start

### Installation
```bash
cargo install skillset
```

### Basic Usage
```bash
# Install React best practices skill
skillset add react-best-practices@1.0.0

# Install from a specific source
skillset add my-skill --source git:https://github.com/user/repo

# List installed skills
skillset list

# Remove a skill
skillset remove react-best-practices
```

### Configuration
Create `skillset.json` in your project:

```json
{
  "skills": {
    "react-best-practices": "1.0.0",
    "@user/web-scraper": "2.1.0",
    "custom-skill": {
      "version": "3.0.0",
      "source": "git:https://github.com/vercel-labs/agent-skills",
      "convention": "agent-skills"
    }
  },
  "registry": "ghcr.io/skillset",
  "conventions": ["autogpt", "langchain", "agent-skills"]
}
```

## Features

- **Multi-Framework Support**: Works with Auto-GPT, LangChain, Vercel Agent Skills, and custom agent frameworks
- **Smart Organization**: Automatically organizes skills by framework conventions
- **Multiple Sources**: Install from Git repositories, OCI registries, or local paths
- **Version Management**: Pin specific versions or use `latest`
- **Scoped Namespaces**: Use `@user/skill` format for community skills
- **Zero-Configuration Caching**: Automatic cross-project skill sharing
- **Production-Ready Skills**: Access Vercel's React best practices and other production-grade skills

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
- **Vercel Agent Skills**: Automatically detected and organized as `skills/agent-skills/{name}/`
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
│   ├── langchain/
│   │   └── llm-tool/
│   └── agent-skills/
│       └── react-best-practices/
└── .skillset/
    └── cache/
```

## Development

For architecture details and integration guidelines, see [AGENTS.md](./AGENTS.md).