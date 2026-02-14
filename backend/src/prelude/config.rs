use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid environment variable: {0}")]
    InvalidEnvVar(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub jwt_secret: String,
    pub database_url: String,
    pub mailjet_api_key: String,
    pub mailjet_api_secret: String,
    pub url: String,
    pub mail_from_email: String,
    pub mail_from_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = std::env::var("PORT")
            .map_err(|_| ConfigError::MissingEnvVar("PORT".to_string()))?
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidEnvVar("PORT".to_string()))?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?;

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

        let mailjet_api_key = std::env::var("MAILJET_API_KEY")
            .map_err(|_| ConfigError::MissingEnvVar("MAILJET_API_KEY".to_string()))?;

        let mailjet_api_secret = std::env::var("MAILJET_API_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("MAILJET_API_SECRET".to_string()))?;

        let frontend_url =
            std::env::var("URL").map_err(|_| ConfigError::MissingEnvVar("URL".to_string()))?;

        let mail_from_email = std::env::var("MAIL_FROM_EMAIL")
            .map_err(|_| ConfigError::MissingEnvVar("MAIL_FROM_EMAIL".to_string()))?;

        let mail_from_name = std::env::var("MAIL_FROM_NAME")
            .map_err(|_| ConfigError::MissingEnvVar("MAIL_FROM_NAME".to_string()))?;

        Ok(Config {
            port,
            jwt_secret,
            database_url,
            mailjet_api_key,
            mailjet_api_secret,
            url: frontend_url,
            mail_from_email,
            mail_from_name,
        })
    }
}
