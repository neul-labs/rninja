---
title: Contributing Guide
description: How to contribute to rninja
tags:
  - contributing
---

# Contributing Guide

Welcome! We're excited you want to contribute to rninja.

## Ways to Contribute

### Report Bugs

Found a bug? Open an issue with:

- rninja version (`rninja --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Minimal test case if possible

### Suggest Features

Have an idea? Open an issue with:

- Use case description
- Proposed solution
- Any alternatives considered

### Improve Documentation

Documentation improvements are always welcome:

- Fix typos and grammar
- Clarify confusing sections
- Add examples
- Translate to other languages

### Submit Code

Ready to code? Follow the process below.

## Development Process

### 1. Find an Issue

- Check [good first issues](https://github.com/anthropics/rninja/labels/good%20first%20issue)
- Look for [help wanted](https://github.com/anthropics/rninja/labels/help%20wanted)
- Or propose your own in an issue

### 2. Fork and Clone

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR-USERNAME/rninja
cd rninja
git remote add upstream https://github.com/anthropics/rninja
```

### 3. Create Branch

```bash
git checkout -b feature/your-feature
# or
git checkout -b fix/issue-123
```

### 4. Make Changes

- Follow the [code style](code-style.md)
- Add tests for new functionality
- Update documentation if needed

### 5. Test

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

### 6. Commit

Write clear commit messages:

```
feat: add support for xyz

Implement xyz feature that allows users to do abc.

Closes #123
```

Format: `type: description`

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructure
- `test`: Tests
- `chore`: Maintenance

### 7. Push and PR

```bash
git push origin feature/your-feature
```

Open a Pull Request with:
- Clear title and description
- Link to related issue
- Screenshots if UI changes

## Pull Request Guidelines

### Before Submitting

- [ ] Tests pass locally
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] Commit messages are clear

### PR Description Template

```markdown
## Summary
Brief description of changes.

## Changes
- Change 1
- Change 2

## Testing
How was this tested?

## Related
Closes #123
```

### Review Process

1. Maintainer reviews within a few days
2. Address feedback with new commits
3. Maintainer approves and merges
4. Your contribution is released!

## Code Review

### What We Look For

- **Correctness**: Does it work as intended?
- **Tests**: Is new code tested?
- **Performance**: Any performance impact?
- **Style**: Follows conventions?
- **Documentation**: Is it documented?

### Responding to Feedback

- All feedback is meant to improve the code
- Ask questions if something is unclear
- Push new commits to address feedback
- Mark conversations as resolved

## Community Guidelines

### Be Respectful

- Assume good intentions
- Be constructive in feedback
- Help newcomers

### Be Patient

- Maintainers are volunteers
- Reviews may take time
- Complex PRs need more review

### Be Collaborative

- Discuss before large changes
- Break large PRs into smaller ones
- Share knowledge

## Recognition

Contributors are:
- Listed in release notes
- Added to CONTRIBUTORS file
- Credited in commit history

## Getting Help

- Open an issue for questions
- Join discussions on GitHub
- Check existing issues and PRs

## Legal

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT).

## Next Steps

- [Development Setup](development-setup.md): Set up your environment
- [Code Style](code-style.md): Coding conventions
- [Testing](testing.md): How to test
- [Roadmap](roadmap.md): Project direction
