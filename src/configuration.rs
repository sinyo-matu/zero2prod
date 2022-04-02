use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub enum Environment {
    Local,
    Production,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_path = base_path.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_path.join("base")).required(true))
        .add_source(
            config::File::from(configuration_path.join(environment.as_str())).required(true),
        )
        .build()?;
    settings.try_deserialize::<Settings>()
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
    pub fn connection_string(&self) -> Secret<String> {
        let DatabaseSettings {
            username,
            password,
            port,
            host,
            database_name,
        } = self;
        Secret::new(format!(
            "postgres://{username}:{password}@{host}:{port}/{database_name}",
            password = password.expose_secret()
        ))
    }
    pub fn connection_string_without_db(&self) -> Secret<String> {
        let DatabaseSettings {
            username,
            password,
            port,
            host,
            database_name: _,
        } = self;
        Secret::new(format!(
            "postgres://{username}:{password}@{host}:{port}",
            password = password.expose_secret()
        ))
    }
}
