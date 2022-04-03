use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration();
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    // init logger
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .expect("Failed to bind port 8000");
    run(listener, connection_pool)?.await
}
