use config::Source;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgSslMode;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};
use tracing::{info, instrument};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

pub enum Environment {
    Local,
    Production,
}

#[instrument(name = "get configuration")]
pub fn get_configuration() -> Settings {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_path = base_path.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let env = config::Environment::with_prefix("app").separator("__");
    let envs = env.collect().expect("failed to collect envs");
    info!("got env: {:?}", &envs);
    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_path.join("base")).required(true))
        .add_source(
            config::File::from(configuration_path.join(environment.as_str())).required(true),
        )
        .add_source(env)
        .build()
        .expect("failed to read config");
    let settings = settings
        .try_deserialize::<Settings>()
        .expect("failed to read config");
    info!("successfully get settings :{:?}", &settings);
    settings
}

impl Environment {
    fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not supported environment. use either `local` or `production` instead.",
                other
            )),
        }
    }
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    #[instrument(name = "load pg connection options", skip(self))]
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        info!(
            "db connection with host: {} username: {} password: {:?} port: {},ssl_mode: {}",
            &self.host, &self.username, &self.password, self.port, self.require_ssl
        );
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}
