# Code Explanation & Documentation Rules

## 1. Comprehensive Explanation
- **Explain Every Block**: When generating code, provide a clear explanation of what each major block does.
- **Why, Not Just What**: Focus on the reasoning behind the implementation choices, not just a line-by-line translation.
- **Contextualize**: Explain how the new code fits into the existing architecture.

## 2. Documentation Standards
- **Public API**: All public functions, structs, traits, and enums MUST have documentation comments (`///`).
- **Complex Logic**: Add inline comments (`//`) to explain complex algorithms, edge cases, or non-obvious logic.
- **Examples**: Where applicable, include usage examples in the documentation to demonstrate intended use.

## 3. Library Usage Rationale
- **Justify Choices**: When using external libraries (e.g., `tokio`, `axum`, `sqlx`, `serde`), explicitly explain *why* specific modules, functions, or patterns were chosen.
  - *Example*: "Used `tokio::sync::mpsc` channel here to handle asynchronous message passing between the worker and the main thread, ensuring non-blocking communication."
- **Deep Dive**: If a specific feature of a library is used (e.g., a specific `axum` extractor or a `serde` attribute), explain its role and why it was necessary for this specific implementation.
