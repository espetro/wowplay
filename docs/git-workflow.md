# Git Workflow: Atomic Commits

## Commit Format
Use **Conventional Commits**: `<type>: <description>`

### Commit Types
- `feat:` New feature
- `fix:` Bug fix
- `refactor:` Code change without functional change
- `test:` Test additions/changes
- `docs:` Documentation changes
- `chore:` Build/config changes
- `ci:` CI/CD changes

### Commit Scopes
Use parentheses to specify scope: `<type>(scope): <description>`

Scopes in this project:
- `rust-core`: Rust domain and adapter changes
- `zig-glue`: Zig/C++ build system changes
- `integration`: Testing and harness changes
- `docs`: Documentation (no scope needed)

## Atomic Commit Guidelines

### 1. One Logical Change Per Commit
Each commit should represent a single, complete thought:
- ✅ Add port trait for x87 translation
- ✅ Implement rosettax87 adapter
- ✅ Add MRE test for FXCH translation
- ❌ Add translation, tests, and documentation (split into 3 commits)

### 2. Commit Should Pass All Checks Independently
Each commit must:
- Pass `cargo check` and `cargo clippy` (if Rust code)
- Pass `cargo fmt --check` (formatted correctly)
- Pass tests (if adding functionality)
- Not break the build

### 3. No WIP Commits in Main Branch
- Use draft PRs for work-in-progress
- Squash WIP commits before merging
- Keep main branch green

### 4. Write Descriptive Commit Messages
- ✅ `feat(rust-core): add RosettaTranslationPort trait for x87 instruction translation`
- ✅ `fix(integration): correct FFI signature for rosettax87_init function`
- ❌ `update stuff` (too vague)
- ❌ `wip` (incomplete)

## Examples

### Feature Addition
```bash
git commit -m "feat(rust-core): add WineIntegrationPort trait

Defines the contract for Wine/CrossOver process management.
Includes methods for process handle retrieval and dylib injection."
```

### Bug Fix
```bash
git commit -m "fix(rust-core): correct buffer size in translate_x87_instruction

Previous buffer size of 128 bytes was insufficient for complex
x87 instructions. Increased to 256 bytes with proper bounds checking."
```

### Test Addition
```bash
git commit -m "test(integration): add MRE test for FXCH instruction translation

Validates that rosettax87_jit correctly translates the FXCH ST(1)
instruction (0xD9 0xC9) to equivalent AArch64 code."
```

### Documentation
```bash
git commit -m "docs: add porting policy decision matrix

Clarifies when and why we port C/C++ libraries to Rust,
with flowchart and examples."
```

### Refactoring
```bash
git commit -m "refactor(rust-core): extract common error handling into utility module

Consolidates error conversion logic from multiple adapters into
a single ffi_error.rs module for better maintainability."
```

## Branch Strategy

### Feature Branches
```bash
# Naming convention: feature/description, fix/description
git checkout -b feature/add-rosetta-port
git checkout -b fix/ffi-bounds-check
```

### Commit Workflow
```bash
# 1. Make changes
git add packages/rust-core/src/ports/rosetta_port.rs

# 2. Check formatting
cargo fmt

# 3. Run checks
./scripts/pre-commit.sh

# 4. Commit
git commit -m "feat(rust-core): add RosettaTranslationPort trait"

# 5. Push and create PR
git push origin feature/add-rosetta-port
```

## Pull Request Guidelines

### PR Title Format
Use same format as commits: `[Type] Description`

Examples:
- `[Feat] Add x87 translation port and adapter`
- `[Fix] Correct FFI bounds checking in rosettax87 wrapper`
- `[Docs] Porting policy documentation`

### PR Description Structure
```markdown
## Summary
- Add X feature
- Fix Y bug
- Update Z documentation

## Testing
- Added MRE test for X
- Verified Y with real WoW process

## Checklist
- [ ] All tests pass
- [ ] Code formatted
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

## Pre-Commit Hook

The `scripts/pre-commit.sh` hook runs `just check`, which enforces:
1. rosettax87_jit freshness (CMake binaries newer than source)
2. Rust formatting (`cargo fmt --check`)
3. Rust linting (`cargo clippy`)
4. GUI linting (`bun run lint`)
5. GUI validation (`bun run validate`)

Run manually: `./scripts/pre-commit.sh`

## Common Mistakes to Avoid

### 1. Mixing Concerns
❌ Bad: Add feature + fix bug + update docs
✅ Good: Separate commits for each

### 2. Broken Intermediate States
❌ Bad: Commit that fails `cargo check`
✅ Good: Each commit builds successfully

### 3. Vague Messages
❌ Bad: "update", "fix", "wip"
✅ Good: Describe what and why

### 4. Large Commits
❌ Bad: 500 lines across 10 files
✅ Good: Break into logical steps

## Rewriting History

### Squashing Commits
```bash
# Interactive rebase to squash last 3 commits
git rebase -i HEAD~3

# Use "squash" or "fixup" to combine commits
# Use "reword" to edit commit messages
```

### Fixing Recent Commits
```bash
# Add forgotten file to last commit
git add forgotten_file.rs
git commit --amend --no-edit

# Fix last commit message
git commit --amend -m "new message"
```

**Warning:** Never amend pushed commits.

## Agent-Specific Workflow

### Before Making Changes
1. Read relevant AGENTS.md
2. Understand scope of change
3. Plan atomic commits

### While Working
1. Run `./scripts/pre-commit.sh` frequently
2. Commit often, commit small
3. Keep changes buildable

### Before Pushing
1. Ensure all checks pass
2. Review commit history
3. Write descriptive PR description

## See Also
- [Architecture](architecture.md) - System design
- [Porting Policy](porting-policy.md) - When to port components
