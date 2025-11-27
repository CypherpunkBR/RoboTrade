# Code Quality Standards for RoboTrade

## Metrics e Metas

### Code Coverage
- **Meta**: >= 70% (ideal: >= 80%)
- **Atual**: Em construção
- **Tool**: cargo-tarpaulin
- **CI**: Automatic reporting to Codecov

### Clippy Warnings
- **Meta**: 0 warnings com `-D warnings`
- **Lints**: all, pedantic, nursery
- **CI**: Bloqueante (build falha se warnings)

### Formatting
- **Tool**: rustfmt
- **Config**: `rustfmt.toml` (2 espaços, 100 chars)
- **CI**: Automatic check (non-blocking fix)

## Quality Gates (CI/CD)

### Pre-merge Requirements

1. **Formatting** ✅
   ```bash
   cargo fmt --all -- --check
   ```

2. **Linting** ✅
   ```bash
   cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all -W clippy::pedantic -W clippy::nursery
   ```

3. **Build** ✅
   ```bash
   cargo check --workspace --all-targets --all-features
   ```

4. **Tests** ✅
   ```bash
   cargo test --workspace --all-features
   ```

5. **Security** ⚠️ (warning only)
   ```bash
   cargo audit
   cargo deny check
   ```

6. **Unused Dependencies** ⚠️ (warning only)
   ```bash
   cargo machete
   ```

## Local Development

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

cargo fmt --all -- --check || {
    echo "❌ Formatting check failed. Run 'cargo fmt --all'"
    exit 1
}

cargo clippy --workspace --all-targets --all-features -- -D warnings || {
    echo "❌ Clippy check failed. Fix warnings or suppress with #[allow(...)]"
    exit 1
}

cargo test --workspace || {
    echo "❌ Tests failed"
    exit 1
}

echo "✅ All pre-commit checks passed"
```

### Quick Quality Check

```bash
# Run before committing
cargo fmt --all
cargo clippy --fix --allow-dirty --allow-staged --workspace --all-targets --all-features
cargo test --workspace
```

## Code Complexity Limits

### Functions
- **Max lines**: 100 (enforced by clippy)
- **Max arguments**: 5 (enforced by clippy)
- **Cognitive complexity**: <= 15 (enforced by clippy)

### Files
- **Recommended max lines**: 500
- **Action if exceeded**: Consider splitting into modules

## Documentation Coverage

### Required
- All `pub` functions
- All `pub` structs and enums
- All modules (`//! Module doc`)

### Format
```rust
/// Brief one-line description.
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function returns an error
///
/// # Examples
///
/// ```
/// let result = my_function("test");
/// assert!(result.is_ok());
/// ```
pub fn my_function(param1: &str, param2: i32) -> Result<String> {
    // ...
}
```

## Test Coverage Requirements

### Unit Tests
- All public functions
- Edge cases (empty inputs, max values, etc.)
- Error cases

### Integration Tests
- Trait implementations (ExchangeGateway, Strategy, etc.)
- Repository patterns
- End-to-end workflows

### Example Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_happy_path() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_error_case() {
        let result = function_under_test(invalid_input);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

## Performance Monitoring

### Benchmarks
- Critical paths (indicator calculation, backtest engine)
- Run with `cargo bench`
- Track regressions

### Profiling
- Flamegraphs para hotspots
- Memory profiling com valgrind
- CI não bloqueia (informativo apenas)

## Security Audits

### Dependencies
```bash
# Run weekly
cargo audit

# Run before releases
cargo deny check advisories
cargo deny check licenses
```

### Code Analysis
- No unsafe code sem justificativa
- Input validation em boundaries
- Secret management via env vars

## Continuous Improvement

### Weekly
- Review clippy lints (add new checks)
- Update dependencies
- Check security advisories

### Monthly
- Review test coverage
- Update documentation
- Refactor high-complexity code

### Per Release
- Full audit run
- Documentation completeness check
- Performance baseline update

---

**Mantenha a qualidade alta desde o início!**
