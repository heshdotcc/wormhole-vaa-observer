# Wormhole VAA Observer

## Rationale
A repository made of a challenge finding missing VAA sequences within Wormhole Scan API,

The ultimate idea is to: 

* Revamp [wormhole-dashboard](https://github.com/wormhole-foundation/wormhole-dashboard) with newer technologies while exploring the [wormhole-sdk](https://wormhole-foundation.github.io/wormhole-sdk-ts/) and its APIs. 
* Understand the status-quo of Wormhole [devnet](https://github.com/wormhole-foundation/wormhole/tree/main/devnet) while looking for improving documentation and its adoption.
* Finding for ways to contribute to [wormhole-sdk-rs](https://github.com/wormhole-foundation/wormhole-sdk-rs) crate maintenance and its potential integrations.
* Integrate a defacto industry-grade workflow engine like Temporal.io to handle VAA ingestion and processing.

## Root Structure

The project consists of three root folders:

* `documentation`  The place of Architectural Design Records (ADRs) and other documentation assets.
* `infrastructure` Kubernetes manifests regarding Wormhole devnet, monitoring, and deployments.
* `microservices`  A backend as the core solution and a hybrid (CSR+SSR) frontend application.

Each folder contains its own `README.md` file for scoped documentation:

```
.
├── README.md
├── documentation
│   ├── ADR-01-DOMAINS.md
│   ├── ADR-02-TECHNOLOGY.md
│   └── ADR-03-VALIDATION.md
├── infrastructure
│   └── wormhole-spy.yaml
│   └── README.md
└── microservices
    ├── backend
    └── frontend
```

## Roadmap
Note that this project was made as a mere proof of concept. Still, ADRs will serve as a future-proof guide.

### Microservices

**BackEnd**
- [x] A production-grade microservice template for a highly scalable backend.
- [x] An integration of Wormhole Scan API to fetch VAAs through a Rust HTTP Server/Client.
- [x] An integration of Wormhole Spy to fetch raw VAAs through a Rust gRPC Server/Client.
- [ ] A robust Anomaly Detection domain use-case to find duplicated and missing VAAs.
  - [x] A rudimentary way to detect duplicated raw VAAs through a local Spy.
  - [ ] Sophistication of raw VAAs decoding algorithm (based on official efforts).
  - [ ] Stored version of missing/duplicated VAAs with cache and persistence.
- [ ] An event store that provides audibility and reproducibility of the domain storage.
- [ ] A transactional database to command and query analytics, enabled with a time-series extension.

**FrontEnd**
- [ ] A Deno-based WebSocket connection to ingest Spy backend data in its low-latency fashion.
- [x] A high-level interface for DataTables, Charts, and LocalStorage customizable Widgets.

### Infrastructure
**Kubernetes Deployment**
  - [x] Deployment of the Wormhole Spy service in a Kubernetes cluster with NodePort exposure.
  - [ ] Integration of a service mesh to secure gRPC and REST communication between the backend and frontend.
**Monitoring and Observability**
  - [ ] Integration of Prometheus and Grafana for backend metrics and logs.
  - [ ] Instrumentation of backend ([WIP](https://github.com/heshdotcc/wormhole-vaa-observer/pull/2)) and frontend services with custom metrics and traces.
  - [ ] Dashboards for Wormhole Spy and VAA analytics (e.g. gRPC latency, request volume, and anomaly detection statistics)
**CI/CD Pipeline**
  - [ ] GitHub Actions or GitLab CI pipelines to automate building, testing, and deployment.
  - [ ] Integration with a local Docker or Kaniko registry for container builds.
  - [ ] Automated rollback mechanisms for Kubernetes deployments using Helm or Argo Rollouts.
**DevOps Utilities**
  - [ ] A configurable Ngrok Kubernete Operator to securely expose the microservices during development.
  - [ ] Terraform IaC templates for setting up the infrastructure in hybrid environments.
  - [ ] Local development tools using Nix Shell and devcontainers.
