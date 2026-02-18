# 🤝 Contributing to BullSharks.online

Thank you for your interest in contributing! This project is designed to be collaboration-friendly for both human developers and AI agents.

## 🚀 Quick Start

### For AI Agents
**Start here**: [🤖 Agent Contribution Guide](/docs/AGENT_CONTRIBUTION_GUIDE.md)

This guide is specifically designed for AI agents and contains everything you need to understand the system and contribute effectively.

### For Human Developers
1. **Read the documentation** - Start with [📚 Documentation Hub](/docs/README.md)
2. **Set up locally** - Follow [Quick Start](#quick-start-setup) below
3. **Understand the workflow** - Review [Development Workflow](/docs/DEVELOPMENT_WORKFLOW.md)
4. **Make your changes** - Follow our [guidelines](#contribution-guidelines)

## 🛠️ Quick Start Setup

```bash
# Clone and setup
git clone https://github.com/BraydenRoyston/bullsharks.online.git
cd bullsharks.online

# Environment setup
cp .env.example .env
# Edit .env with your configuration

# Install and test
cargo build
cargo test

# Start development server
cargo run
```

## 📋 Contribution Guidelines

### Before You Start
- [ ] Read the [Agent Contribution Guide](/docs/AGENT_CONTRIBUTION_GUIDE.md) or [System Architecture](/docs/SYSTEM_ARCHITECTURE.md)
- [ ] Check existing issues for similar work
- [ ] Understand the [Code Organization](/docs/CODE_ORGANIZATION.md)

### Making Changes
- [ ] Create a feature branch: `git checkout -b feature/descriptive-name`
- [ ] Follow the [Development Workflow](/docs/DEVELOPMENT_WORKFLOW.md)
- [ ] Write/update tests following [Testing Strategy](/docs/TESTING_STRATEGY.md)
- [ ] Update relevant documentation (see [Documentation Requirements](#documentation-requirements))

### Before Submitting
- [ ] All tests pass: `cargo test`
- [ ] Code compiles cleanly: `cargo clippy`
- [ ] Code is formatted: `cargo fmt`
- [ ] Documentation is updated

## 📝 Documentation Requirements

**Critical**: Update documentation with every change to ensure the next contributor has accurate context.

### What to Update
- **API changes** → `docs/API_DOCUMENTATION.md`
- **System changes** → `docs/SYSTEM_ARCHITECTURE.md`
- **New modules** → `docs/CODE_ORGANIZATION.md`
- **Database changes** → `docs/DATABASE_SCHEMA.md`
- **New tests** → `docs/TESTING_STRATEGY.md`

## 🎯 Types of Contributions

### 🐛 Bug Fixes
- Check [Troubleshooting Guide](/docs/TROUBLESHOOTING.md) first
- Include reproduction steps
- Add tests to prevent regression

### ✨ New Features
- Start with system understanding ([System Architecture](/docs/SYSTEM_ARCHITECTURE.md))
- Follow existing patterns ([Code Organization](/docs/CODE_ORGANIZATION.md))
- Include comprehensive tests

### 📚 Documentation
- All documentation improvements welcome
- Keep the "living documentation" philosophy
- Update with every code change

### 🧪 Tests
- Follow [Testing Strategy](/docs/TESTING_STRATEGY.md)
- Place tests in `src/tests/` directory
- Include both happy path and error cases

## 🔄 Pull Request Process

### PR Template
```markdown
## Summary
Brief description of changes and motivation

## Changes Made
- [ ] Feature/fix implemented
- [ ] Tests added/updated
- [ ] Documentation updated

## Testing
- [ ] `cargo test` passes
- [ ] Manual testing completed
- [ ] Edge cases considered

## Documentation Updates
- [ ] Relevant .md files updated
- [ ] Code comments added where needed

## Breaking Changes
- None / List any breaking changes
```

### Review Process
1. **Automated checks** - CI runs tests, linting, formatting
2. **Documentation review** - Ensure docs are updated
3. **Code review** - Maintainer reviews implementation
4. **Testing validation** - Verify tests cover new functionality
5. **Merge** - Squash and merge after approval

## 🏗️ Project Structure

```
bullsharks.online/
├── docs/                    # 📚 Comprehensive documentation
│   ├── README.md           # Documentation hub
│   ├── AGENT_CONTRIBUTION_GUIDE.md  # AI agent guide
│   ├── SYSTEM_ARCHITECTURE.md      # System design
│   ├── CODE_ORGANIZATION.md        # Code structure
│   ├── DATABASE_SCHEMA.md          # Data models
│   ├── DEVELOPMENT_WORKFLOW.md     # Development process
│   ├── TESTING_STRATEGY.md         # Testing approach
│   └── TROUBLESHOOTING.md          # Problem solving
├── src/                     # 🦀 Rust source code
│   ├── api/                # HTTP endpoints (thin layer)
│   ├── services/           # Business logic (thick layer)
│   ├── models/             # Data structures
│   ├── tests/              # Test suites
│   └── utils/              # Helper functions
├── README.md               # Project overview
└── CONTRIBUTING.md         # This file
```

## 🤖 AI Agent Collaboration

This project is designed with AI agent collaboration in mind:

### For AI Agents
- **Rich Context**: Comprehensive documentation provides system understanding
- **Clear Patterns**: Consistent code organization and conventions
- **Living Docs**: Documentation stays current with code changes
- **Agent Guide**: Specific guide tailored for AI contribution patterns

### Documentation Philosophy
- **Always Current**: Updated with every PR
- **Agent-Friendly**: Written for maximum clarity and context
- **Comprehensive**: Covers both concepts and implementation details
- **Actionable**: Provides clear guidance for making changes

## 🎨 Code Standards

### Rust Conventions
- Follow standard Rust naming: `snake_case`, `PascalCase`, `SCREAMING_SNAKE_CASE`
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Add documentation for public APIs

### Error Handling
```rust
// Good: Provide context
.map_err(|e| ApiError::DatabaseError(format!("Failed to fetch athlete {}: {}", athlete_id, e)))?

// Good: Handle all cases
match result {
    Ok(data) => process_data(data),
    Err(e) => {
        log::error!("Operation failed: {}", e);
        return Err(ApiError::from(e));
    }
}
```

### Time Handling
```rust
// Good: UTC for storage and calculations
let utc_time = Utc::now();

// Good: Convert to display timezone only at boundaries
let pacific_time = Los_Angeles.from_utc_datetime(&utc_time.naive_utc());
```

## 🚨 What Not to Do

- ❌ Don't skip updating documentation
- ❌ Don't mix time zones in business logic
- ❌ Don't add tests inline with source code (use `src/tests/`)
- ❌ Don't ignore compiler warnings
- ❌ Don't make breaking changes without discussion
- ❌ Don't commit directly to main branch

## 🏆 Recognition

Contributors who follow these guidelines help make the project better for everyone:
- **Humans**: Clear code and documentation for future developers
- **AI Agents**: Rich context for effective collaboration
- **Users**: Reliable, well-tested functionality

## 🔗 Resources

- **Documentation Hub**: [docs/README.md](/docs/README.md)
- **System Overview**: [docs/SYSTEM_ARCHITECTURE.md](/docs/SYSTEM_ARCHITECTURE.md)
- **Development Process**: [docs/DEVELOPMENT_WORKFLOW.md](/docs/DEVELOPMENT_WORKFLOW.md)
- **Troubleshooting**: [docs/TROUBLESHOOTING.md](/docs/TROUBLESHOOTING.md)

## 🤔 Questions?

If you have questions or need clarification:
1. Check the [Troubleshooting Guide](/docs/TROUBLESHOOTING.md)
2. Review the relevant documentation
3. Create an issue with your question

Thank you for contributing to BullSharks.online! 🦈🏃‍♂️