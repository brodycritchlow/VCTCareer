# GitHub Actions Local Testing Guide

This guide shows you how to test GitHub Actions locally before committing to ensure they will run successfully.

## 🚀 Quick Start

### 1. Validate Workflow Syntax (Fastest)
```bash
# Validate all workflows for syntax errors
./scripts/test-github-actions.sh validate
```

### 2. Test Individual Components
```bash
# Test backend unit tests (fast, no database)
./scripts/test-github-actions.sh backend-unit

# Test frontend CI
./scripts/test-github-actions.sh frontend

# Test change detection logic
./scripts/test-github-actions.sh changes
```

### 3. Test Full Workflows (Slower)
```bash
# Test complete PR workflow
./scripts/test-github-actions.sh pr-full

# Test main branch workflow
./scripts/test-github-actions.sh main
```

## 🛠️ Tools Available

### Option 1: Act (Recommended for Full Testing)
**Act** runs GitHub Actions locally using Docker containers.

```bash
# List all available jobs
act --list

# Run specific job
act pull_request --job backend-unit-tests

# Run with custom architecture (M1/M2 Macs)
act pull_request --container-architecture linux/amd64
```

**Pros:**
- ✅ Most accurate simulation of GitHub Actions
- ✅ Tests actual Docker containers and dependencies
- ✅ Can test complex workflows with services (PostgreSQL)

**Cons:**
- ❌ Requires Docker (can be slow)
- ❌ Large download for runner images (~GB)
- ❌ May not work perfectly on M1/M2 Macs

### Option 2: ActionLint (Recommended for Syntax Validation)
**ActionLint** validates workflow syntax without running them.

```bash
# Install actionlint
brew install actionlint

# Validate all workflows
actionlint .github/workflows/*.yml
```

**Pros:**
- ✅ Very fast (seconds)
- ✅ Catches syntax errors and common mistakes
- ✅ No Docker required
- ✅ Perfect for pre-commit validation

**Cons:**
- ❌ Doesn't test actual execution
- ❌ Can't catch runtime errors

### Option 3: Pre-commit Hooks (Recommended for Automation)
Automatically validates workflows before commits.

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files

# Test GitHub Actions validation specifically
pre-commit run test-github-actions --all-files
```

## 🎯 Recommended Workflow

### Before Committing Changes:

1. **Always run syntax validation** (fastest):
   ```bash
   ./scripts/test-github-actions.sh validate
   ```

2. **Test affected components**:
   ```bash
   # If you changed backend code
   ./scripts/test-github-actions.sh backend-unit
   
   # If you changed frontend code  
   ./scripts/test-github-actions.sh frontend
   
   # If you changed workflow files
   ./scripts/test-github-actions.sh changes
   ```

3. **Optional: Full test** (before major changes):
   ```bash
   ./scripts/test-github-actions.sh pr-full
   ```

### Setting Up Pre-commit (One-time setup):
```bash
# Install pre-commit
pip install pre-commit

# Install actionlint for workflow validation
brew install actionlint

# Install the pre-commit hooks
pre-commit install

# Test the setup
pre-commit run --all-files
```

## 🚨 Common Issues and Solutions

### 1. Docker Issues on M1/M2 Macs
```bash
# Use x86 architecture
act --container-architecture linux/amd64

# Or add to .actrc file (already configured)
echo "--container-architecture linux/amd64" >> .actrc
```

### 2. PostgreSQL Service Issues
The workflows are optimized to only use PostgreSQL when needed:
- **Unit tests**: No PostgreSQL (fast)
- **Integration tests**: With PostgreSQL (slower)

```bash
# Test only unit tests (no DB required)
act pull_request --job backend-unit-tests

# Test integration (requires Docker PostgreSQL)
act pull_request --job backend-integration-tests
```

### 3. Environment Variables
Local testing uses `.env.act` file:
```bash
# Edit environment variables for testing
nano .env.act
```

### 4. Workflow File Syntax Errors
```bash
# Validate specific workflow
actionlint .github/workflows/pr-ci.yml

# Get detailed error information
act --list --workflows .github/workflows/pr-ci.yml
```

## 📊 Performance Comparison

| Method | Speed | Accuracy | Setup Required |
|--------|-------|----------|----------------|
| ActionLint | ⚡ Seconds | 🎯 Syntax only | ✅ Simple |
| Act (unit tests) | 🔄 ~2-3 min | 🎯 High | 🐳 Docker |
| Act (full workflow) | 🐌 ~10-15 min | 🎯 Very High | 🐳 Docker + PostgreSQL |
| Pre-commit hooks | ⚡ Seconds | 🎯 Syntax + Format | ✅ One-time setup |

## 🎛️ Configuration Files

- **`.actrc`**: Act configuration (architecture, artifacts)
- **`.env.act`**: Environment variables for local testing
- **`.pre-commit-config.yaml`**: Pre-commit hook configuration
- **`scripts/test-github-actions.sh`**: Custom testing script

## 💡 Best Practices

1. **Always validate syntax** before committing
2. **Test unit tests locally** for backend changes
3. **Use pre-commit hooks** for automated validation
4. **Run full workflows** only for major changes
5. **Check logs carefully** when tests fail locally

## 🔧 Troubleshooting

### Act doesn't start:
```bash
# Check Docker is running
docker ps

# Clean up containers
docker system prune -f

# Update act
brew upgrade act
```

### Pre-commit hooks fail:
```bash
# Update hooks
pre-commit autoupdate

# Clear cache
pre-commit clean

# Reinstall
pre-commit uninstall && pre-commit install
```

### Workflow changes not detected:
```bash
# Check git status
git status

# Ensure workflow files are tracked
git add .github/workflows/
```

## 📞 Quick Reference

```bash
# Common commands
./scripts/test-github-actions.sh validate     # Syntax check
./scripts/test-github-actions.sh backend-unit # Fast backend test
./scripts/test-github-actions.sh list         # Show all jobs
./scripts/test-github-actions.sh help         # Show help

# Direct act usage
act --list                                     # List jobs
act pull_request --job backend-unit-tests     # Run specific job
act push --workflows .github/workflows/main-ci.yml  # Run workflow

# Pre-commit
pre-commit run --all-files                    # Run all hooks
pre-commit run actionlint --all-files         # Just workflow validation
```