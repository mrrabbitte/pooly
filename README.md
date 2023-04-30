[<img src="https://upload.wikimedia.org/wikipedia/commons/thumb/e/ea/Presidential_Standard_of_Belarus_%28fictional%29.svg/240px-Presidential_Standard_of_Belarus_%28fictional%29.svg.png" width="20" height="20" alt="Voices From Belarus" />](https://voicesfrombelarus.org/) [![Stand With Ukraine](https://raw.githubusercontent.com/vshymanskyy/StandWithUkraine/main/badges/StandWithUkraine.svg)](https://vshymanskyy.github.io/StandWithUkraine)

[![Dependency Review](https://github.com/mrrabbitte/pooly/actions/workflows/dependency-review.yml/badge.svg)](https://github.com/mrrabbitte/pooly/actions/workflows/dependency-review.yml) [![Rust](https://github.com/mrrabbitte/pooly/actions/workflows/rust.yml/badge.svg)](https://github.com/mrrabbitte/pooly/actions/workflows/rust.yml)

# pooly

A protobuf to Postgres adapter + connection pooling middleware.

## Project status

WIP. The roadmap includes:

- [x] Batch requests + trasactions support;
- [x] basic unit tests;
- [x] basic integration tests;
- [x] send back bytes received from db, accept strongly typed param values;
- [x] JWT auth + admin / client_service roles;
- [x] wider range of postgres types support as input and output;
- [x] PBT + basic integration tests coverage;
- [x] throttle requests based on the client_id;
- [ ] OpenTelemetry support;
- [ ] Java client;
- [ ] add option to use Vault instead of local secrets keeping;
- [ ] Docs on pooly;
- [ ] rich logging support;
- [ ] docker image + unsealing script;
- [ ] add validation to config / admin value objects;
- [ ] more test coverage;
- [ ] TLS postgres support;
- [ ] Python client;
- [ ] Rust client;
- [ ] even wider range of postgres types;
- [ ] queries deduping.

