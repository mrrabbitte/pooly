# pooly

A protobuf to Postgres adapter + connection pooling middleware.

## Project status

WIP. The roadmap includes:

- [x] Batch requests + trasactions support;
- [x] basic unit tests;
- [x] basic integration tests;
- [ ] send back bytes received from db, accept strongly typed param values;
- [ ] JWT auth + admin / client_service roles;
- [ ] Client based throttling;
- [ ] TLS postgres support;
- [ ] Java client;
- [ ] Property-Based Testing;
- [ ] Python client;
- [ ] Rust client;
- [ ] rich logging support;
- [ ] metrics support -> Prometheus;
- [ ] wide range of postgres types support as input;
- [ ] add option to use Vault instead of local secrets keeping;
- [ ] docker image + unsealing script.
