# Code Explanation & Documentation Rules

## 1. Comprehensive Explanation
- **Explain Every Block**: When generating code, provide a clear explanation of what each major block does.
- **Why, Not Just What**: Focus on the reasoning behind the implementation choices, not just a line-by-line translation.
- **Contextualize**: Explain how the new code fits into the existing architecture.

## 2. Documentation Standards (STRICT ENFORCEMENT)
- **Mandatory Logic Explanation**: Every function and method MUST have a preceding documentation comment (`///` or `//`) explaining its **Purpose**, **Parameters**, and **Return Value**.
- **Preserve Existing Comments**: When modifying existing code, you MUST NOT delete existing helpful comments unless they are obsolete. If you rewrite logic, rewrite the corresponding comments.
- **No Silent Code Drops**: Do not remove documentation during refactoring. If a function is complex, add inline comments explaining the *why* of the logic steps.
- **Self-Documenting Code**: While code should be readable, always supplement it with comments that explain the *intent* and *business logic*, not just syntax.
- **Examples**: Where applicable, include usage examples in the documentation to demonstrate intended use.

## 3. Library Usage Rationale
- **Justify Choices**: When using external libraries (e.g., `tokio`, `axum`, `sqlx`, `serde`), explicitly explain *why* specific modules, functions, or patterns were chosen.
  - *Example*: "Used `tokio::sync::mpsc` channel here to handle asynchronous message passing between the worker and the main thread, ensuring non-blocking communication."
- **Deep Dive**: If a specific feature of a library is used (e.g., a specific `axum` extractor or a `serde` attribute), explain its role and why it was necessary for this specific implementation.
