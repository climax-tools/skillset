# AGENTS.md - Skillset CLI Architecture and Integration Guide

## Tool Overview

**Skillset** is a Rust-based CLI package manager designed specifically for coding agent skills. It provides npm-like semantics for managing skills across multiple coding agent frameworks while abstracting away the complexity of different agent conventions.

### Scope and Purpose

- **Primary Goal**: Simplify skill discovery, installation, and management for AI agent developers
- **Target Audience**: Node.js/TypeScript developers, Python developers working with AI agents
- **Use Cases**: Development workflows, CI/CD pipelines, skill sharing, and agent ecosystem integration

### Key Problems Solved

1. **Fragmented Agent Ecosystem**: Different frameworks (Auto-GPT, LangChain, custom) have incompatible skill formats
2. **Complex Installation**: No unified way to install skills across frameworks
3. **Configuration Management**: Projects need to track skills, dependencies, and framework compatibility
4. **Distribution**: No standard way to package and share agent skills

### Design Constraints

- **Framework Agnostic**: Must work with existing and future agent frameworks
- **Developer Friendly**: Familiar CLI patterns and configuration formats
- **Extensibility**: Easy to add new frameworks, sources, and features
- **Production Ready**: Robust error handling, configuration management, and caching

## Table of Contents

1. [Core Architecture Overview](#core-architecture-overview)
2. [Pluggable Sources System](#pluggable-sources-system)
3. [Convention System Architecture](#convention-system-architecture)
4. [Configuration Format Migration](#configuration-format-migration)
5. [Project Structure Patterns](#project-structure-patterns)
6. [CLI Design and Semantics](#cli-design-and-semantics)
7. [Integration Guidelines for Agent Frameworks](#integration-guidelines-for-agent-frameworks)
8. [Development Guidelines](#development-guidelines)
9. [Code Examples and Patterns](#code-examples-and-patterns)
10. [Architecture Decision Rationale](#architecture-decision-rationale)

---

## Core Architecture Overview

Skillset follows a modular architecture with clear separation of concerns:

### Key Design Principles

1. **Orthogonal Configuration**: Agent conventions are configured separately from skill definitions
2. **Pluggable Extensibility**: Both sources and conventions can be easily extended
3. **CLI-First Design**: Command-line interface drives all operations
4. **Async-First**: All I/O operations are asynchronous

### Module Organization

```
├── src/
│   ├── cli/                    # CLI interface and command handling
│   ├── sources/                 # Pluggable source implementations
│   ├── conventions/              # Agent framework conventions
│   ├── config/                  # Configuration management
│   ├── skill/                   # Core skill data structures
│   ├── registry/                 # OCI registry operations
│   └── error.rs                # Centralized error handling
└── AGENTS.md                   # This architectural documentation
```

---

## Pluggable Sources System

### SkillSource Trait (`src/sources/mod.rs`)

All skill sources implement the `SkillSource` trait:

```rust
#[async_trait]
pub trait SkillSource: Send + Sync {
    async fn fetch(&self, reference: &str) -> Result<FetchedSkill>;
    async fn get_metadata(&self, reference: &str) -> Result<SkillMetadata>;
    fn source_type(&self) -> SourceType;
}
```

#### Supported Source Types

1. **Git Sources** (`src/sources/git.rs`)
   - Clone repositories from GitHub/GitLab/Gitea
   - Parse git URLs and references
   - Handle branches and tags
   - Extract skill content from repository root

2. **OCI Sources** (`src/sources/oci.rs`)
   - Pull from OCI registries (Docker Hub, GitHub Container Registry, etc.)
   - Support ORAS-like artifact format
   - Handle authentication and layer manifests
   - Extract skill content from OCI layers

3. **Local Sources** (`src/sources/local.rs`)
   - Load skills from local filesystem paths
   - Useful for development and testing
   - Symlink or copy content to organized directories

#### Source Registry Pattern

```rust
pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn SkillSource>>,
}

impl SourceRegistry {
    pub fn register(&mut self, source: Box<dyn SkillSource>) {
        let source_type = source.source_type();
        let type_name = match source_type {
            SourceType::Git => "git",
            SourceType::Oci => "oci",
            SourceType::Local => "local",
        };
        self.sources.insert(type_name.to_string(), source);
    }
}
```

---

## Convention System Architecture

### Convention Trait (`src/conventions.rs`)

All agent framework conventions implement the `Convention` trait:

```rust
#[async_trait]
pub trait Convention: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    async fn detect(&self, path: &std::path::Path) -> Result<bool>;
    async fn organize(&self, skill_name: &str, source_path: &std::path::Path, target_path: &std::path::Path) -> Result<()>;
    fn config(&self) -> &ConventionConfig;
}
```

#### Built-in Conventions

1. **Auto-GPT Convention** (`src/conventions/autogpt.rs`)
   - **Detection Patterns**: `["skill.py", "requirements.txt", "__init__.py"]`
   - **Path Pattern**: `skills/autogpt/{name}/`
   - **Metadata File**: `skill.json`
   - **Structure**: Main skill file + dependencies

2. **LangChain Convention** (`src/conventions/langchain.rs`)
   - **Detection Patterns**: `["tool.yaml", "*.py", "pyproject.toml"]`
   - **Path Pattern**: `skills/langchain/{name}/`
   - **Metadata File**: `tool.yaml`
   - **Structure**: Tool definition with Python implementation

3. **Custom Convention** (`src/conventions/custom.rs`)
   - **Detection Patterns**: `["*.js", "package.json", "index.js"]`
   - **Path Pattern**: `skills/custom/{name}/`
   - **Metadata File**: `package.json`
   - **Structure**: Node.js package with entry point

#### Convention Registry

```rust
pub struct ConventionRegistry {
    conventions: HashMap<String, Box<dyn Convention>>,
}
```

---

## Configuration Format Simplification

### JSON-First Minimalist Configuration

**Rationale**: Simplified configuration for minimal happy path with automatic reference resolution and reduced cognitive overhead.

**Key Design Decisions**:
- **No Backward Compatibility**: Clean slate approach with simplified schema
- **Automatic Reference Resolution**: Simple names map to well-known OCI registry
- **Scoped Name Support**: `@user/skill` format for user-scoped skills
- **Flexible Skill Values**: Simple version strings or detailed metadata objects

**Simplified Configuration Schema** (`src/config/skillset.rs`):
```json
{
  "skills": {
    "file-analyzer": "1.0.0",
    "@johndoe/web-scraper": "2.1.0",
    "complex-skill": {
      "version": "3.0.0",
      "source": "git:https://github.com/custom/repo",
      "convention": "autogpt"
    }
  },
  "registry": "ghcr.io/skillset",
  "conventions": ["autogpt", "langchain"]
}
```

#### Reference Resolution Logic

**Simple Skills**:
- `"file-analyzer": "1.0.0"` → `oci:ghcr.io/skillset/file-analyzer:v1.0.0`
- Uses project `registry` as base (e.g., `ghcr.io/skillset`)

**Scoped Skills**:
- `"@johndoe/web-scraper": "2.1.0"` → `oci:ghcr.io/johndoe/web-scraper:v2.1.0`
- Username extracted from scope, registry domain maintained from project config

**Complex Skills**:
- `"complex-skill": { ... }` → Uses explicit `source` field or falls back to same resolution
- Allows overriding convention and source

#### Skill Configuration Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SkillConfig {
    Simple(String), // Just version
    Detailed {
        version: String,
        source: Option<String>, // Override auto-resolution
        convention: Option<String>, // Override auto-detection
    }
}
```

---

## Project Structure Patterns

### Skill Organization by Convention

Skills are organized according to their detected or specified convention. The simplified configuration removes complex convention settings in favor of auto-detection:

```
project/
├── skillset.json              # Simplified configuration file
├── skills/                    # Auto-organized skills by framework
│   ├── autogpt/             # Auto-GPT framework skills (auto-detected)
│   │   ├── file-analyzer/   # skill.py, requirements.txt, skill.json
│   │   └── web-scraper/      # skill.py, requirements.txt, skill.json
│   ├── langchain/            # LangChain framework skills (auto-detected)
│   │   ├── llm-tool/         # tool.yaml, tool.py, pyproject.toml
│   │   └── document-summarizer/ # tool.yaml, llm_tool.py
│   └── custom/             # Custom framework skills (user-specified)
│       └── my-tool/        # package.json, index.js
└── .skillset/               # Working directory
    ├── cache/                # Downloaded repositories
    │   ├── file-analyzer/   # Cached source code
    │   └── web-scraper/    # Cached source code
    └── metadata/              # Extracted skill metadata
```

**Auto-Detection Priority**:
1. **Convention Override**: Explicit `convention` field in complex skill config
2. **File Pattern Detection**: Built-in detection logic for each framework
3. **Fallback**: Default to custom convention if no patterns match

**Simplified Convention Management**:
- Conventions are enabled by name in `conventions` array
- No complex path patterns or detection patterns in config
- All detection logic is hardcoded for simplicity and performance

---

## CLI Design and Semantics

### npm-like Semantics

The CLI follows familiar npm package manager semantics with simplified resolution:

```bash
# Install simple skill (auto-resolves to registry)
skillset add file-analyzer@1.0.0

# Install scoped skill (user namespace)
skillset add @johndoe/web-scraper@2.0.0

# Install from explicit source
skillset add custom-skill --source git:https://github.com/user/repo

# Install with convention override
skillset add my-tool --convention langchain

# Install from OCI registry (explicit)
skillset add oci:ghcr.io/user/skill:v1.0.0

# List all installed skills
skillset list

# List with verbose output
skillset list --verbose

# Remove a skill
skillset remove skill-name

# Update skills
skillset update [skill-name]

# Get skill information
skillset info skill-name

# Manage conventions
skillset convention list
skillset convention enable autogpt
skillset convention disable langchain

# Publish to OCI registry
skillset publish ./my-skill oci:ghcr.io/user/my-skill:v1.0.0
```

#### Simplified Reference Resolution

**Skill Name Resolution**:
- `file-analyzer` → `oci:ghcr.io/skillset/file-analyzer:v1.0.0`
- `@johndoe/web-scraper` → `oci:ghcr.io/johndoe/web-scraper:v2.0.0`
- Custom sources override auto-resolution

**Version Handling**:
- Versions are automatically prefixed with `v` for OCI tags
- `latest` resolves to the most recent version in registry

### Command Structure (`src/cli/mod.rs`)

```rust
#[derive(Subcommand)]
pub enum Commands {
    Add { reference: String, convention: Option<String>, version: Option<String> },
    Remove { name: String },
    List { verbose: bool },
    Update { name: Option<String> },
    Info { name: String },
    Convention { command: ConventionCommands },
    Publish { path: String, reference: String, registry: Option<String> },
}
```

### Reference Parsing Logic (`src/skill/manager.rs`)

The system parses references to determine source type:

- **Git URLs**: `git:https://github.com/user/repo`
- **OCI References**: `oci:ghcr.io/user/skill:v1.0.0`
- **Local Paths**: `./local-skill` or `/absolute/path/to/skill`

---

## Integration Guidelines for Agent Frameworks

### Adding New Frameworks

To integrate a new agent framework:

1. **Create Convention Implementation** (`src/conventions/my_framework.rs`)
   ```rust
   impl Convention for MyFrameworkConvention {
       fn name(&self) -> &str { "my-framework" }
       fn detect(&self, path: &Path) -> Result<bool> { /* detection logic */ }
       fn organize(&self, skill_name: &str, source_path: &Path, target_path: &Path) -> Result<()> { /* organization logic */ }
       fn description(&self) -> &str { "My framework description" }
       fn version(&self) -> &str { "1.0.0" }
   }
   ```

2. **Register Convention in SkillManager** (`src/skill/manager.rs`)
   ```rust
   let mut manager = SkillManager::new(project_path)?;
   manager.convention_registry.register(Box::new(MyFrameworkConvention::new()));
   ```

3. **Add to Enabled Conventions** (in project `skillset.json`)
   ```json
   {
     "conventions": ["autogpt", "langchain", "my-framework"]
   }
   ```

**Simplified Framework Integration**: 
- Detection logic is hardcoded in the convention implementation
- No complex configuration patterns in JSON
- Convention name used for directory organization (`skills/my-framework/{name}/`)

### Framework Integration Example

**Auto-GPT Integration**:
```python
# skills/autogpt/my-skill/skill.json
{
  "name": "my-skill",
  "description": "A skill for Auto-GPT",
  "entry_point": "skill.py",
  "dependencies": ["requests", "openai"]
}
```

**LangChain Integration**:
```python
# skills/langchain/my-tool/tool.yaml
name: my-tool
description: A LangChain-compatible tool
tool_type: llm_function
input_schema:
  type: object
  properties:
    query:
      type: string
      description: The input query
```

---

## Development Guidelines

### Code Organization

1. **Trait-Based Design**: Use traits for pluggable components
2. **Error Handling**: Comprehensive error types with proper `From` implementations
3. **Async/Await**: Use async traits and `.await` for I/O operations
4. **Configuration Management**: Centralized config loading and saving

### Adding New Features

1. **Trait Implementation**: Always implement required trait methods
2. **Error Variants**: Create specific error variants for each failure mode
3. **Configuration Structures**: Add new fields to appropriate structs
4. **CLI Commands**: Extend `Commands` and `ConventionCommands` enums
5. **Testing**: Add tests for new functionality

### Documentation Requirements

1. **Code Comments**: Document public APIs and complex logic
2. **Examples**: Provide usage examples for each new feature
3. **README Updates**: Update feature lists and integration guides
4. **Architectural Decisions**: Document non-obvious decisions in this file

---

## Code Examples and Patterns

### Skill Source Implementation Example

```rust
pub struct GitSource {
    client: git2::Repository,
}

#[async_trait]
impl SkillSource for GitSource {
    async fn fetch(&self, reference: &str) -> Result<FetchedSkill> {
        let repo = git2::Repository::clone(reference, ".skillset/cache/skill-name")?;

        // Extract metadata
        let version = self.get_git_version(&repo)?;
        let source_path = repo.path();

        Ok(FetchedSkill {
            name: "skill-name".to_string(),
            version,
            source_path,
            metadata: SkillMetadata {
                installed_at: chrono::Utc::now().to_rfc3339(),
                repo_path: source_path.display().to_string(),
                convention: "detected".to_string(), // Will be detected later
                checksum: None,
            },
        })
    }

    async fn get_metadata(&self, reference: &str) -> Result<SkillMetadata> {
        // Implementation for remote metadata fetching
        todo!("Implement git metadata fetching")
    }

    fn source_type(&self) -> SourceType {
        SourceType::Git
    }
}
```

### Convention Implementation Example

```rust
pub struct AutoGptConvention {
    config: ConventionConfig,
}

#[async_trait]
impl Convention for AutoGptConvention {
    async fn organize(&self, skill_name: &str, source_path: &Path, target_path: &Path) -> Result<()> {
        let skill_dir = target_path
            .join("skills")
            .join("autogpt")
            .join(skill_name);

        std::fs::create_dir_all(&skill_dir)?;

        // Copy skill files
        copy_dir_all(source_path, &skill_dir)?;

        // Create requirements file if not exists
        if !skill_dir.join("requirements.txt").exists() {
            std::fs::write(skill_dir.join("requirements.txt"), "requests\nopenai")?;
        }

        Ok(())
    }
}
```

---

## Architecture Decision Rationale

### Why Minimal Configuration?

1. **Reduced Cognitive Load**: Simple skill references with automatic resolution
2. **Developer Experience**: `skillset add file-analyzer@1.0.0` just works without complex setup
3. **Faster Adoption**: Minimal configuration barrier to entry
4. **OCI-Native**: Leverages existing container registry ecosystem

### Why JSON over TOML?

1. **Ecosystem Alignment**: JSON is native to Node.js/TypeScript development
2. **Tool Compatibility**: Better integration with npm, yarn, and existing tooling
3. **Developer Experience**: Familiar format reduces learning curve
4. **Schema Validation**: JSON Schema support for better validation

### Why Automatic Reference Resolution?

1. **Simplicity**: `file-analyzer` → `oci:ghcr.io/skillset/file-analyzer:v1.0.0`
2. **Scoped Namespacing**: `@user/skill` maps naturally to OCI registry structure
3. **Consistency**: All skills follow the same naming and resolution pattern
4. **Flexibility**: Complex skills can still override defaults when needed

### Why Orthogonal Conventions?

1. **Framework Independence**: Skills can work with any agent framework
2. **Flexible Organization**: Different frameworks have different needs
3. **Clear Separation**: Convention logic is separate from skill management
4. **Extensibility**: Easy to add new frameworks without affecting existing skills

### Why Plugin-Based Architecture?

1. **Runtime Extensibility**: Load conventions and sources dynamically
2. **Decoupling**: Core system doesn't depend on specific implementations
3. **Testing**: Easy to test individual components
4. **Multiple Implementations**: Support multiple conventions for the same framework

### Why Async-First?

1. **Performance**: Non-blocking I/O for better user experience
2. **Modern Rust**: Leverages async/await ecosystem
3. **Error Handling**: Proper propagation of async errors
4. **Scalability**: Concurrent operations where possible

## Summary of Simplified Architecture

The simplified architecture prioritizes developer experience and adoption through:

1. **Minimal Configuration**: JSON schema with automatic reference resolution
2. **Zero-Configuration Skills**: Simple name-to-OCI mapping with version management
3. **Scoped Namespaces**: Natural `@user/skill` format for community contributions
4. **Flexible Override**: Complex skills can override conventions and sources when needed
5. **Hardcoded Conventions**: Detection logic in code rather than configuration for performance

This architecture provides a solid foundation for building a comprehensive skill management system that can adapt to various coding agent frameworks while maintaining clean separation of concerns, extensibility, and a developer-first approach to skill distribution and management.
