# Language Runtime Separation

## Status
Accepted

## Context
The execution engine needs to support multiple programming languages. Each language has its own compilation and execution requirements.

## Decision
We will separate the language-specific logic (compilation and execution preparation) from the generic execution engine by defining a LanguageRuntime trait. Each language will implement this trait to provide:
- How to prepare the compilation step (if needed)
- How to prepare the execution step
- What files are expected

## Consequences

### Positive
- Easy to add new languages by implementing the LanguageRuntime trait
- Execution engine remains language-agnostic
- Clear separation of concerns
- Compilation and execution can be independently sandboxed (if desired)

### Negative
- Slight indirection in the execution pipeline
- Need to define a comprehensive trait that covers all language needs