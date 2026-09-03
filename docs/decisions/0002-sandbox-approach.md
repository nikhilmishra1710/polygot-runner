# Sandbox Approach: Linux Primitives vs Docker

## Status
Accepted

## Context
The platform needs to securely execute untrusted code across multiple programming languages. Traditional approaches often use Docker containers for isolation, but this adds complexity and overhead.

## Decision
We will build the sandbox directly on Linux primitives (namespaces, cgroups, seccomp, mount isolation) rather than relying on Docker or other containerization technologies.

## Consequences

### Positive
- Reduced attack surface (no daemon running as root)
- Lower resource overhead (no container engine)
- Faster startup times
- More granular control over isolation mechanisms
- Better alignment with "Linux-first design" principle
- Avoids dependency on Docker daemon availability

### Negative
- Increased implementation complexity
- Need to handle more low-level details manually
- Potential for security mistakes in isolation implementation
- Less portable to non-Linux systems (though Linux was target anyway)

## Related Decisions
- 0003-language-runtime-separation: Separation of compilation and execution phases