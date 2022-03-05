# pooly

A protobuf to Postgres adapter + connection pooling middleware.

## Project status

WIP. The roadmap includes:

- [x] Batch requests + trasactions support;
- [x] basic unit tests;
- [x] basic integration tests;
- [x] send back bytes received from db, accept strongly typed param values;
- [ ] JWT auth + admin / client_service roles;
- [ ] throttle requests based on the client_id;
- [ ] wide range of postgres types support as input and output + make returning raw bytes as specified by the request;
- [ ] TLS postgres support;
- [ ] gRPC Streaming results
- [ ] add validation to config / admin value objects;
- [ ] Java client;
- [ ] Property-Based Testing + increased test coverage for all of the services;
- [ ] Python client;
- [ ] Rust client;
- [ ] rich logging support;
- [ ] metrics support -> Prometheus;

- [ ] add option to use Vault instead of local secrets keeping;
- [ ] docker image + unsealing script.

