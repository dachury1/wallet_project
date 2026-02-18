---
description: Workflow for checking test coverage of the project using cargo-llvm-cov.
---

# Test Coverage Workflow

This workflow ensures that the codebase maintains a high level of test coverage, critical for the financial reliability of the Wallet/Transaction system.

## 1. Prerequisites Check
- Verify that `cargo-llvm-cov` is installed.
- **Action**: Check if `cargo llvm-cov --version` runs successfully.
- **Fallback**: If not installed, suggest running `cargo install cargo-llvm-cov` or `rustup component add llvm-tools-preview`.

## 2. Prepare Environment
- Ensure clean state to avoid stale coverage data.
- **Command**: `cargo clean -p <package_name>` if necessary, but usually `cargo llvm-cov clean` handles this.

## 3. Execute Coverage Analysis
// turbo
- Run the full test suite with instrumentation.
- **Command**: `cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info` (generates report).
- **Command**: `cargo llvm-cov report` (shows summary in terminal).

## 4. Analysis
- Review the coverage summary printed in the terminal.
- **Target**: Aim for >80% coverage in Domain logic.
- Identify files with low coverage (especially in `domain/` or `use_cases/`).

## 5. Report Generation (Optional)
- If detailed inspection is needed, generate an HTML report.
- **Command**: `cargo llvm-cov --html`
- **Output**: Open `target/llvm-cov/html/index.html` in browser.

## 6. Cleanup
- Remove temporary coverage artifacts if they are large.
