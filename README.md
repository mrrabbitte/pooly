# pooly

A protobuf to Postgres adapter + connection pooling middleware.

## Project status

WIP. The roadmap includes:

- [x] Batch requests + trasactions support;
- [x] basic unit tests;
- [x] basic integration tests;
- [ ] JWT auth + admin / client_service roles;
- [ ] you can throttle requests based on the client-id;
- [ ] TLS postgres support;
- [ ] Java client;
- [ ] Property-Based Testing;
- [ ] Python client;
- [ ] Rust client;
- [ ] rich logging support;
- [ ] metrics support -> Prometheus;
- [ ] wide range of postgres types support as input;
- [ ] add option to use Vault instead of local secrets keeping;
- [ ] docker image + unsealing script;
- [ ] send back bytes received from db, accept strongly typed param values;
