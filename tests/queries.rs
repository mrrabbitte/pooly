use testcontainers::{clients, core::WaitForMessage, Docker, images::postgres::Postgres};

#[tokio::test]
async fn test_single_query() {
    let _ = pretty_env_logger::try_init().unwrap();
    let docker = clients::Cli::default();

    let container = docker.run(Postgres::default());

    let pg_host = container.get_host_port(5432);

    
}