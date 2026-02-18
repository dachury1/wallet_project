---
description: Workflow for implementing new features safely using TDD and strict Rust guidelines.
---

# Feature Implementation Workflow

This workflow ensures that every new feature in the Wallet/Transaction system is robust, tested, and follows the project's strict safety guidelines.

## 1. Analysis & Design
- Review the user's requirements for the new feature.
- Identify which microservice (Wallet or Transaction) and which layer (Domain, Use Case, Infrastructure) is involved.
- **Rule Check**: Recall the "General Rust Development Rules" (No `unwrap`, use `Result`, etc.).

## 2. Create Failing Test (Red)
- Before writing any production code, create a new test case that defines the expected behavior.
- If it's pure logic, put it in `mod tests` within the file.
- If it's an integration flow, put it in `tests/`.
- Run the test to confirm it fails (or fails to compile because the code doesn't exist yet).

## 3. Implementation (Green)
- Write the minimal amount of code necessary to make the test pass.
- Use `todo!()` macros if you need to sketch out the structure first.
- **Constraint**: Do NOT use `unwrap()` or `expect()`. Use `?` for error propagation.

## 4. Refactor & Polish (Blue)
- Review the code for :
    - **Safety**: Are all errors handled?
    - **Performance**: Are there unnecessary `.clone()` calls? Could we use references?
    - **Idiomatic Rust**: Can a `for` loop be a functional iterator chain?
- Run `cargo clippy` to catch common mistakes.
- Ensure variable naming is clear and follows Rust conventions.

## 5. Verification
// turbo
- Run `cargo test` to ensure the new test passes and no regressions were introduced.
- Run `cargo clippy -- -D warnings` to ensure code quality.

## 6. Documentation
- Add `///` doc comments to any new public structs, enums, or functions.
- Include an `# Examples` section in the doc comment if the usage isn't obvious.
