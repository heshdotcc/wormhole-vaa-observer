# ADR-01: Technology Choices

## Context
Building a modern observability platform requires technology choices that balance performance, maintainability, and developer experience.

## Decisions

### HTTP Frameworks: Axum & Hyper

Based on [recent benchmarks](https://web-frameworks-benchmark.netlify.app/compare?f=express,gin,axum) (as of Jan 2024):

| Language (version) | Framework (version) | Requests/Second (64) | Requests/Second (256) | Requests/Second (512) |
|-------------------|--------------------|--------------------|---------------------|---------------------|
| rust (1.82)       | axum (0.7)        | 375,750           | 522,988            | 543,514            |
| go (1.23)         | gin (1.10)        | 332,406           | 359,419            | 360,668            |
| javascript        | express (4.21)     | 88,007            | 86,523             | 85,261             |

Key benefits:
- High performance with tokio async runtime.
- Type-safe routing with compile-time checks.
- Latest OpenAPI docs support with Scalar.

### gRPC Framework: Tonic

Selected for Wormhole Spy integration:
- Bi-directional streaming for real-time VAA data.
- Protocol Buffers for efficient serialization.
- Seamless integration with Axum (both Tower-based).
- Native support for OpenTelemetry instrumentation.

### Observability Stack

1. **OpenTelemetry**
   - Industry standard open-source solution for unified traces, metrics, and logs.
   - There's a yet-working upcoming PoC for Prometheus metrics with a Grafana dashboard.
   - A branch `feature/observability` is under progress for OTEL logs tracing and metrics with Grafana Tempo.

2. **Health Checks**
   - Follows [Kubernetes health endpoints convention](https://kubernetes.io/docs/reference/using-api/health-checks/) (`/healthz`, `/livez`, `/readyz`)
   - Handles application lifecycle (DB, cache, message queue readiness)

### Project Structure
Domain-driven design with repository pattern for domain/data abstraction:
```
src/
├── domain/           # Business logic by domain
│   ├── health/       # Health checks domain
│   └── wormhole/     # Wormhole domain (VAA handling)
├── library/          # Shared utilities
├── storage/          # Repository pattern implementation
│   ├── database.rs   # Database abstraction
│   └── memory.rs     # In-memory implementation
└── main.rs           # Application entry point
```

A bit of facade pattern was done for storage abstraction, still to be extended with Postgres and TimescaleDB.

*TODO: Implement an event store and the CQRS pattern for command/query separation.*

## Consequences

### Positive
- High performance and low resource usage.
- Strong type safety and compile-time guarantees.
- Excellent observability integration.
- Efficient real-time data streaming.

### Challenges
- Steeper learning curve for Rust newcomers.
- Smaller ecosystem vs Node.js/Go.
- Protocol-specific error handling needed.

## References
1. [Web Framework Benchmarks](https://web-frameworks-benchmark.netlify.app/compare?f=express,gin,axum)
2. [Axum Repository](https://github.com/tokio-rs/axum)
3. [OpenTelemetry](https://opentelemetry.io/)
4. [Tonic gRPC](https://github.com/hyperium/tonic)
5. [Wormhole Protobufs](https://github.com/wormhole-foundation/wormhole/tree/main/proto)
6. [Kubernetes Health Checks](https://kubernetes.io/docs/reference/using-api/health-checks/) 