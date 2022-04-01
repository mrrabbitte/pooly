[<img src="https://upload.wikimedia.org/wikipedia/commons/thumb/e/ea/Presidential_Standard_of_Belarus_%28fictional%29.svg/240px-Presidential_Standard_of_Belarus_%28fictional%29.svg.png" width="20" height="20" alt="Voices From Belarus" />](https://voicesfrombelarus.org/) [![Stand With Ukraine](https://raw.githubusercontent.com/vshymanskyy/StandWithUkraine/main/badges/StandWithUkraine.svg)](https://vshymanskyy.github.io/StandWithUkraine)

# pooly

A protobuf to Postgres adapter + connection pooling middleware.

## Project status

WIP. The roadmap includes:

- [x] Batch requests + trasactions support;
- [x] basic unit tests;
- [x] basic integration tests;
- [x] send back bytes received from db, accept strongly typed param values;
- [x] JWT auth + admin / client_service roles;
- [ ] wider range of postgres types support as input and output;
- [ ] TLS postgres support;
- [ ] add validation to config / admin value objects;
- [ ] Java client;
- [ ] docker image + unsealing script;
- [ ] Property-Based Testing + increased test coverage for all of the services;
- [ ] throttle requests based on the client_id;
- [ ] gRPC Streaming results
- [ ] Python client;
- [ ] Rust client;
- [ ] rich logging support;
- [ ] metrics support -> Prometheus;
- [ ] even wider range of postgres types + optionally return raw bytes;
- [ ] add option to use Vault instead of local secrets keeping;
- [ ] queries deduping.

