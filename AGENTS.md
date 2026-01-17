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
4. [Project Structure Patterns](#project-structure-patterns)
5. [CLI Design and Semantics](#cli-design-and-semantics)
6. [Integration Guidelines for Agent Frameworks](#integration-guidelines-for-agent-frameworks)
7. [Development Guidelines](#development-guidelines)
8. [Code Examples and Patterns](#code-examples-and-patterns)
9. [Architecture Decision Rationale](#architecture-decision-rationale)

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
│   ├── cache/                   # User-wide cache infrastructure
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
   - **User-Wide Cache**: Clone to `~/.cache/skillset/git/checkouts/{skill-name}/`
   - **Metadata Tracking**: JSON files in `~/.cache/skillset/metadata/`
   - Parse git URLs and references
   - Handle branches and tags
   - Extract skill content from repository root
   - **Cache-Aware**: Initialize with shared `CachePaths` infrastructure

2. **OCI Sources** (`src/sources/oci.rs`) - *Not Yet Implemented*
   - Pull from OCI registries (Docker Hub, GitHub Container Registry, etc.)
   - Support ORAS-like artifact format
   - Handle authentication and layer manifests
   - Extract skill content from OCI layers
   - **Future Extensibility**: Cache infrastructure ready for OCI sources

3. **Local Sources** (`src/sources/local.rs`) - *Not Yet Implemented*
   - Load skills from local filesystem paths
   - Useful for development and testing
   - Symlink or copy content to organized directories
   - **Future Extensibility**: Can implement caching with shared CachePaths

#### Source Registry Pattern

```rust
pub struct SourceRegistry {
    sources: HashMap<String, Box<dyn SkillSource>>,
}

impl SourceRegistry {
    pub fn new() -> Result<Self> {
        let mut sources = HashMap::new();
        
        // Sources are cache-aware from initialization
        sources.insert("git".to_string(), Box::new(GitSource::new()?) as Box<dyn SkillSource>);
        
        Ok(Self { sources })
    }
    
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

**Key Changes**:
- **Built-in Sources**: Initialize cache-aware sources automatically
- **No Project Dependencies**: SourceRegistry works independently
- **Error Propagation**: Cache initialization errors bubble up

---

## User-Wide Cache Architecture

### Cache-First Design Principle

**Decision**: Sources manage both retrieval AND caching internally, rather than having a separate cache management layer. This unified approach simplifies architecture while enabling user-wide deduplication.

**Benefits**:
- **Cross-Project Skill Sharing**: Same skill cached once, used by multiple projects
- **No Cache Configuration**: Zero-configuration caching that works automatically  
- **Extensible Pattern**: New source types can implement custom caching strategies
- **Platform Compliance**: Follows OS-standard cache directory conventions

### User-Wide Cache Structure

```
~/.cache/skillset/                    # Platform-appropriate user cache
├── git/                              # Git source cache
│   ├── db/                         # Bare repositories (content-addressable)
│   │   └── {sha256-url-hash}/   # Named by cache key
│   └── checkouts/                   # Working directories  
│       └── {skill-name}/            # Named by skill
└── metadata/                          # Cache metadata files
    └── {sha256-key}.json            # Cache entry metadata
```

#### Cache Key Strategy

**Content-Addressable Keys**: SHA256 hash of `url#reference` format
```rust
fn git_cache_key(url: &str, reference: Option<&str>) -> String {
    let input = format!("{}#{}", url, reference.unwrap_or("latest"));
    format!("{:x}", Sha256::digest(input.as_bytes()))
}
```

**Key Benefits**:
- **Automatic Deduplication**: Same URL+reference always maps to same cache key
- **Collision Resistance**: Cryptographic hash prevents naming conflicts
- **Reference Isolation**: Different branches/tags create separate cache entries
- **Extensible Format**: Supports future `url#branch#commit` patterns

### Cache Infrastructure (`src/cache/mod.rs`)

#### CachePaths - Centralized Path Management

```rust
pub struct CachePaths {
    base_dir: PathBuf,      // ~/.cache/skillset
    git_dir: PathBuf,       // ~/.cache/skillset/git
    metadata_dir: PathBuf,   // ~/.cache/skillset/metadata
}

impl CachePaths {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::cache_dir()?.join("skillset");
        // Platform-agnostic cache directory creation
    }
    
    pub fn ensure_directories(&self) -> Result<()> {
        // Automatic creation of cache structure
        std::fs::create_dir_all(&self.git_dir.join("db"))?;
        std::fs::create_dir_all(&self.git_dir.join("checkouts"))?;
        std::fs::create_dir_all(&self.metadata_dir)?;
    }
}
```

**Key Features**:
- **Platform Detection**: Uses `dirs::cache_dir()` for cross-platform compatibility
- **Automatic Setup**: Creates required directories on first use
- **Type Safety**: Path builders prevent string concatenation errors
- **Clone Support**: CachePaths is Clone for async task spawning

### Cache Metadata System (`src/cache/metadata.rs`)

#### CacheMetadata - Simple JSON Persistence

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub url: String,              // Original source URL
    pub reference: Option<String>, // Branch, tag, or commit
    pub skill_name: String,        // Extracted skill name
    pub source_type: String,       // "git", "oci", "local"
}

impl CacheMetadata {
    pub async fn load(path: &Path) -> Result<Option<Self>>;
    pub async fn save(&self, path: &Path) -> Result<()>;
}
```

**Design Decisions**:
- **JSON Format**: Human-readable and version control friendly
- **Async I/O**: Non-blocking file operations throughout
- **Automatic Parent Creation**: Handles nested directory creation
- **Comprehensive Error Handling**: Clear error messages for debugging

#### Metadata Benefits

- **Cache Validation**: Future can check if cache entry matches current remote
- **Size Management**: Metadata enables cleanup algorithms
- **Source Tracking**: Track original URL and reference for debugging
- **Type Awareness**: Supports multi-source cache strategies

### Source-Local Caching Pattern

#### Unified Source-Cache Implementation

```rust
impl SkillSource for GitSource {
    pub fn new() -> Result<Self> {
        let cache = CachePaths::new()?;
        cache.ensure_directories()?;
        Ok(Self { cache })
    }
    
    async fn fetch(&self, reference: &str) -> Result<FetchedSkill> {
        // Source manages both retrieval AND caching
        let checkout_path = self.get_or_clone(&url, reference, &skill_name).await?;
        // Cache metadata is saved automatically
        Ok(FetchedSkill { /* ... */ })
    }
}
```

**Key Patterns**:
- **No Project Dependencies**: Sources initialize without project path
- **Self-Contained Caching**: Each source manages its cache strategy
- **Shared Infrastructure**: All sources use common CachePaths
- **Atomic Operations**: Cache updates are atomic via temp files

#### Cache Management Flow

1. **Cache Key Generation**: `SHA256(url#reference)`
2. **Checkout Creation**: `git/checkouts/{skill-name}/`  
3. **Metadata Persistence**: `metadata/{cache-key}.json`
4. **Error Recovery**: Cleanup on failed operations
5. **Return Ready**: FetchedSkill with cached content path

### Integration Architecture

#### SourceRegistry Updates

```rust
impl SourceRegistry {
    pub fn new() -> Result<Self> {
        let mut sources = HashMap::new();
        
        // Sources are cache-aware from initialization
        sources.insert("git".to_string(), Box::new(GitSource::new()?) as Box<dyn SkillSource>);
        
        Ok(Self { sources })
    }
}
```

**Changes**:
- **Built-in Sources**: Initialize cache-aware sources automatically
- **No Project Dependencies**: SourceRegistry works independently
- **Error Propagation**: Cache initialization errors bubble up

#### SkillManager Decoupling

```rust
impl SkillManager {
    pub fn new(project_path: PathBuf) -> Result<Self> {
        // Sources manage their own caching
        let source_registry = SourceRegistry::new()?;
        
        Ok(Self {
            convention_registry,
            config,
            project_path,
            source_registry,
        })
    }
}
```

**Benefits**:
- **Zero Cache Management**: SkillManager doesn't handle cache operations
- **Clean Separation**: Orchestration vs. storage concerns separated
- **Consistent Behavior**: All skills use same caching infrastructure

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

2. **LangChain Convention** (`src/conventions/langchain.rs`)
   - **Detection Patterns**: `["tool.yaml", "*.py", "pyproject.toml"]`
   - **Path Pattern**: `skills/langchain/{name}/`
   - **Metadata File**: `tool.yaml`

3. **Custom Convention** (`src/conventions/custom.rs`)
   - **Detection Patterns**: `["*.js", "package.json", "index.js"]`
   - **Path Pattern**: `skills/custom/{name}/`
   - **Metadata File**: `package.json`

#### Convention Registry

```rust
pub struct ConventionRegistry {
    conventions: HashMap<String, Box<dyn Convention>>,
}
```

---

## Project Structure Patterns

### Skill Organization by Convention

Skills are organized according to their detected or specified convention:

```
project/
├── skillset.json              # Configuration file
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

---

## CLI Design and Semantics

### npm-like Semantics

The CLI follows familiar npm package manager semantics:

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

### Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum SkillsetError {
    #[error("Source not found: {0}")]
    SourceNotFound(String),
    
    #[error("Cache operation failed: {0}")]
    CacheError(#[from] std::io::Error),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

### Testing Approach

- **Unit Tests**: Test individual trait implementations
- **Integration Tests**: Test CLI commands and end-to-end workflows
- **Mock Sources**: Use mock sources for testing without network dependencies

---

## Code Examples and Patterns

### Unified Git Source with User-Wide Cache

```rust
pub struct GitSource {
    cache: CachePaths,
}

impl GitSource {
    pub fn new() -> Result<Self> {
        let cache = CachePaths::new()?;
        cache.ensure_directories()?;
        Ok(Self { cache })
    }
    
    async fn get_or_clone(&self, url: &str, reference: Option<&str>, skill_name: &str) -> Result<PathBuf> {
        let cache_key = self.cache.git_cache_key(url, reference);
        let checkout_path = self.cache.git_checkout_path(skill_name);
        
        // Remove existing checkout and clone to cache location
        if checkout_path.exists() {
            std::fs::remove_dir_all(&checkout_path)?;
        }
        
        Repository::clone(url, &checkout_path)?;
        
        // Save metadata for cache tracking
        let metadata = CacheMetadata {
            url: url.to_string(),
            reference: reference.map(|r| r.to_string()),
            skill_name: skill_name.to_string(),
            source_type: "git".to_string(),
        };
        
        let metadata_path = self.cache.metadata_path(&cache_key);
        metadata.save(&metadata_path).await?;
        
        Ok(checkout_path)
    }
}

#[async_trait]
impl SkillSource for GitSource {
    async fn fetch(&self, reference: &str) -> Result<FetchedSkill> {
        let (url, ref_spec) = self.parse_reference(reference)?;
        let skill_name = self.extract_skill_name_from_url(&url)?;
        let checkout_path = self.get_or_clone(&url, ref_spec.as_deref(), &skill_name).await?;
        
        Ok(FetchedSkill {
            name: skill_name,
            version: ref_spec.unwrap_or_else(|| "latest".to_string()),
            source_path: checkout_path.clone(),
            metadata: SkillMetadata { /* ... */ },
        })
    }
    
    fn source_type(&self) -> SourceType {
        SourceType::Git
    }
}
```

**Key Patterns**:
- **Cache-Aware Initialization**: Sources use `CachePaths::new()` for shared cache access
- **Atomic Operations**: Cache updates use temp files and atomic renames
- **Metadata Persistence**: Automatic cache metadata saving for tracking
- **Error Recovery**: Cleanup on failed operations to prevent corruption
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
