use google_cloud_storage::client::Storage;
use std::env;

/// Struct Config for setup environment variables
#[derive(PartialEq, Debug, Clone)]
pub struct Config {
    /// Service Endpoint
    /// Default to localhost.
    /// Usually set to 127.0.0.1 or 0.0.0.0.
    pub svc_endpoint: String,
    /// Service Port
    /// Listening port of service.
    /// Default to 8080.
    /// Usually set to 80, 443, 3000, or 8080.
    pub svc_port: String,
    /// Log Level
    /// From `tracing:Level`.
    /// Default to INFO.
    /// Set to DEBUG for development. Usually set to INFO or WARN in production.
    pub log_level: tracing::Level,
    /// Environment
    /// Type of environment.
    /// Default to `Release`. Can be `Development` or `Release`.
    pub environment: Environment,
    /// Data Source
    /// Source of all data (blogs, talks, etc).
    /// Default to `memory`. Available types are `memory`, `sqlite`, and `turso`.
    /// `sqlite` and `turso` required DATABASE_URL envar to be set.
    /// `turso` required TURSO_AUTH_TOKEN envar to be set.
    pub data_source: String,
    /// Secrets
    /// Collection of secrets. Can be load from environment variables or
    /// Google Cloud Storage (GSM)
    pub secrets: Secrets,
    /// Google Cloud Storage (GCS) Bukcet Name (Optional; Secret)
    /// Secret GCS bucket name. Required `SECRETS_OBJECT` to use GCS
    /// as Secret source.
    /// Example: my-secret-bucket
    /// Default to None.
    pub secrets_bucket: Option<String>,
    /// Google Cloud Storage (GCS) Secret Object Name (Optional)
    /// GCS object name of secret file. An alternative to load secretive envars
    /// other than Google Secret Manager.
    /// Required `SECRETS_BUCKET` to use GCS as Secret source.
    /// If set, will override config secrets.
    /// List of secrets:
    /// - JWT_SECRET
    /// - DATABASE_URL
    /// - TURSO_AUTH_TOKEN
    /// - BUCKET_NAME
    ///
    /// Example: secret/my-secret
    /// Default to None.
    pub secrets_object: Option<String>,
    /// Cache Type (Optional)
    /// Type of cache to be used such as `InMemory`.
    /// Default to None
    pub cache_type: Option<Cache>,
    /// Cache Time to Live (Optional)
    /// A cached entry will be expired after the specified duration (s)
    /// past from cache insertion.
    /// Example: 3600
    /// Default to None
    pub cache_ttl: Option<i64>,
}

/// Environment Type
#[derive(PartialEq, Debug, Clone)]
pub enum Environment {
    Development,
    Release,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Cache Type
#[derive(PartialEq, Debug, Clone)]
pub enum Cache {
    InMemory,
}

impl std::fmt::Display for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Collection of secrets
#[derive(PartialEq, Debug, Clone)]
pub struct Secrets {
    /// JWT Secret (Secret)
    /// Secret to encode JWT in authenticated-pages.
    /// Default to `secret` but highly advised to provide your own value.
    pub jwt_secret: String,
    /// Database URL (Optional; Secret)
    /// Database URL (or Path). **Required** if you use `sqlite` or `turso`
    /// as DATA_SOURCE.
    /// Example:
    ///     - sqlite:husni-portfolio.db
    ///     - libsql://husni-portfolio.asia.turso.io
    /// Default to None.
    pub database_url: Option<String>,
    /// Turso Auth Token (Optional; Secret)
    /// Authentication token for turso database. **Required** if you use
    /// `turso` as DATA_SOURCE..
    /// Default to None
    pub turso_auth_token: Option<String>,
}

impl Default for Config {
    /// By default running on localhost:8080 in release
    /// with log-level info and data from memory
    fn default() -> Self {
        let svc_endpoint: String = "localhost".to_string();
        let svc_port: String = "8080".to_string();
        let log_level = tracing::Level::INFO;
        let environment = Environment::Release;
        let data_source: String = "memory".to_string();
        let jwt_secret: String = "secret".to_string();

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
            data_source,
            secrets: Secrets {
                jwt_secret,
                database_url: None,
                turso_auth_token: None,
            },
            secrets_bucket: None,
            secrets_object: None,
            cache_type: None,
            cache_ttl: None,
        }
    }
}

impl Config {
    /// Setup config from environment variables
    pub async fn from_envar() -> Self {
        // Required
        let svc_endpoint: String = env::var("SVC_ENDPOINT")
            .expect("Failed to load SVC_ENDPOINT environment variable. Double check your config");
        let svc_port: String = env::var("SVC_PORT")
            .expect("failed to load SVC_PORT environment variable. Double check your config");
        let log_level = Self::parse_log_level();
        let environment = Self::parse_environment();
        let data_source = Self::parse_data_source();

        // Optional
        let cache_type = Self::parse_cache_type();
        let cache_ttl = Self::parse_optional("CACHE_TTL").map(|c| {
            c.parse::<i64>()
                .expect("Failed to parse CACHE_TTL from String to i64")
        });
        let secrets_bucket = Self::parse_optional("SECRETS_BUCKET");
        let secrets_object = Self::parse_optional("SECRETS_OBJECT");

        // Check SECRETS_BUCKET and SECRETS_OBJECT
        // If set, try to load the secret then override all secrets.
        let (jwt_secret, database_url, turso_auth_token) = if secrets_bucket.is_some()
            && secrets_object.is_some()
        {
            println!("Pulling secrets from Google Cloud Storage");
            let secrets = Self::load_gcs_secrets(
                &secrets_bucket.clone().unwrap(),
                &secrets_object.clone().unwrap(),
            )
            .await;

            (
                secrets.jwt_secret,
                secrets.database_url,
                secrets.turso_auth_token,
            )
        } else {
            // Required Secrets
            let jwt_secret = env::var("JWT_SECRET")
                .expect("failed to load JWT_SECRET environment variable. Double check your config");
            // Optional Secrets
            let database_url = Self::parse_optional("DATABASE_URL");
            let turso_auth_token = Self::parse_optional("TURSO_AUTH_TOKEN");

            (jwt_secret, database_url, turso_auth_token)
        };

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
            data_source,
            secrets: Secrets {
                jwt_secret,
                database_url,
                turso_auth_token,
            },
            secrets_bucket,
            secrets_object,
            cache_type,
            cache_ttl,
        }
    }
    async fn load_gcs_secrets(secrets_bucket: &str, secrets_object: &str) -> Secrets {
        let client = Storage::builder()
            .build()
            .await
            .expect("Failed to build GCS client");
        let mut reader = client
            .read_object(
                format!("projects/_/buckets/{}", &secrets_bucket),
                secrets_object,
            )
            .send()
            .await
            .expect("Failed to read secret object");
        let mut contents = Vec::new();
        while let Some(chunk) = reader
            .next()
            .await
            .transpose()
            .expect("Failed to read chunk")
        {
            contents.extend_from_slice(&chunk);
        }
        let data = String::from_utf8(contents.clone()).unwrap();
        let mut jwt_secret = String::new();
        let mut database_url: Option<String> = None;
        let mut turso_auth_token: Option<String> = None;

        for secret in data.split("\n").collect::<Vec<&str>>() {
            if secret.split_once("=").is_none() {
                continue;
            }
            let (key, value) = secret.split_once("=").unwrap();
            let secret_v = value.to_string();
            match key {
                "JWT_SECRET" => jwt_secret = secret_v,
                "DATABASE_URL" => database_url = Some(secret_v),
                "TURSO_AUTH_TOKEN" => turso_auth_token = Some(secret_v),
                _ => {
                    println!("Unused secret {} is detected.", &key)
                }
            }
        }

        Secrets {
            jwt_secret,
            database_url,
            turso_auth_token,
        }
    }
    /// Parse Optional environment variables
    fn parse_optional(envar: &str) -> Option<String> {
        match env::var(envar) {
            Err(e) => {
                println!(
                    "Failed to load {} environment variable. Set default to None. Error {}",
                    &envar, e
                );
                None
            }
            Ok(val) => match val.as_str() {
                "" => None,
                _ => Some(val),
            },
        }
    }
    /// Parse Log Level
    fn parse_environment() -> Environment {
        match env::var("ENVIRONMENT") {
            Err(e) => {
                println!(
                "Failed to load ENVIRONMENT environment variable. Set default to 'Release'. Error {e}"
            );
                Environment::Release
            }
            Ok(val) => match val.as_str() {
                "release" | "Release" | "RELEASE" => Environment::Release,
                "development" | "Development" | "DEVELOPMENT" => Environment::Development,
                _ => Environment::Release,
            },
        }
    }
    /// Parse Log Level
    fn parse_log_level() -> tracing::Level {
        match env::var("LOG_LEVEL") {
            Err(e) => {
                println!(
                "Failed to load LOG_LEVEL environment variable. Set default to 'info'. Error {e}"
            );
                tracing::Level::INFO
            }
            Ok(val) => match val.as_str() {
                "ERROR" => tracing::Level::ERROR,
                "WARN" => tracing::Level::WARN,
                "INFO" => tracing::Level::INFO,
                "DEBUG" => tracing::Level::DEBUG,
                "TRACE" => tracing::Level::TRACE,
                _ => tracing::Level::INFO,
            },
        }
    }
    /// Parse Data Source
    fn parse_data_source() -> String {
        match env::var("DATA_SOURCE") {
            Err(e) => {
                println!(
                "Failed to load DATA_SOURCE environment variable. Set default to 'memory'. Error {e}"
                );
                "memory".to_string()
            }
            Ok(val) => match val.as_str() {
                "memory" | "sqlite" | "turso" => val,
                _ => {
                    println!("Data Source type {val} is not supported! Default to 'memory'.");
                    "memory".to_string()
                }
            },
        }
    }
    /// Parse Cache Level
    fn parse_cache_type() -> Option<Cache> {
        match env::var("CACHE_TYPE") {
            Err(e) => {
                println!(
                "Failed to load CACHE_TYPE environment variable. Set default to 'None'. Error {e}"
            );
                None
            }
            Ok(val) => match val.as_str() {
                "inmemory" | "InMemory" => Some(Cache::InMemory),
                _ => None,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let log_level = tracing::Level::INFO;
        let environment = Environment::Release;
        let data_source = "memory";
        let jwt_secret = "secret";

        let result = Config::default();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.data_source, data_source);
        assert_eq!(result.secrets.jwt_secret, jwt_secret);
        assert_eq!(result.secrets.database_url, None);
        assert_eq!(result.secrets.turso_auth_token, None);
        assert_eq!(result.secrets_bucket, None);
        assert_eq!(result.secrets_object, None);
        assert_eq!(result.cache_type, None);
        assert_eq!(result.cache_ttl, None);
    }

    #[tokio::test]
    async fn test_from_envar_without_optionals() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let log_level = tracing::Level::INFO;
        let expected_log_level = tracing::Level::INFO;
        let environment = Environment::Release;
        let expected_environment = Environment::Release;
        let data_source = "";
        let expected_data_source = "memory";
        let jwt_secret = "secret";

        set_envars(Config {
            svc_endpoint: svc_endpoint.to_string(),
            svc_port: svc_port.to_string(),
            log_level,
            environment,
            data_source: data_source.to_string(),
            secrets: Secrets {
                jwt_secret: jwt_secret.to_string(),
                database_url: None,
                turso_auth_token: None,
            },
            secrets_bucket: None,
            secrets_object: None,
            cache_type: None,
            cache_ttl: None,
        });

        let result = Config::from_envar().await;

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, expected_log_level);
        assert_eq!(result.environment, expected_environment);
        assert_eq!(result.data_source, expected_data_source);
        assert_eq!(result.secrets.jwt_secret, jwt_secret);
        assert_eq!(result.secrets.database_url, None);
        assert_eq!(result.secrets.turso_auth_token, None);
        assert_eq!(result.secrets_bucket, None);
        assert_eq!(result.secrets_object, None);
        assert_eq!(result.cache_type, None);
        assert_eq!(result.cache_ttl, None);

        remove_envars()
    }

    #[tokio::test]
    async fn test_from_envar_with_optionals() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let expected_log_level = tracing::Level::INFO;
        let environment = Environment::Development;
        let data_source = "sqlite";
        let jwt_secret = "secret";
        let database_url = Some("libsql://husni-portfolio.asia.turso.io".to_string());
        let turso_auth_token = Some("turso_token_123456".to_string());
        let secrets_bucket = Some("".to_string());
        let secrets_object = Some("".to_string());
        let cache_type = Some(Cache::InMemory);
        let cache_ttl = Some(3600_i64);

        set_envars(Config {
            svc_endpoint: svc_endpoint.to_string(),
            svc_port: svc_port.to_string(),
            log_level: tracing::Level::INFO,
            environment: Environment::Development,
            data_source: data_source.to_string(),
            secrets: Secrets {
                jwt_secret: jwt_secret.to_string(),
                database_url: database_url.clone(),
                turso_auth_token: turso_auth_token.clone(),
            },
            secrets_bucket,
            secrets_object,
            cache_type: cache_type.clone(),
            cache_ttl,
        });

        let result = Config::from_envar().await;

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, expected_log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.data_source, data_source);
        assert_eq!(result.secrets.jwt_secret, jwt_secret);
        assert_eq!(result.secrets.database_url, database_url);
        assert_eq!(result.secrets.turso_auth_token, turso_auth_token);
        assert_eq!(result.secrets_bucket, None);
        assert_eq!(result.secrets_object, None);
        assert_eq!(result.cache_type, cache_type);
        assert_eq!(result.cache_ttl, cache_ttl);

        remove_envars()
    }

    fn set_envars(config: Config) {
        let empty = "";

        env::set_var("SVC_ENDPOINT", config.svc_endpoint);
        env::set_var("SVC_PORT", config.svc_port);
        env::set_var("LOG_LEVEL", config.log_level.to_string());
        env::set_var("ENVIRONMENT", config.environment.to_string());
        env::set_var("DATA_SOURCE", config.data_source);
        env::set_var("JWT_SECRET", config.secrets.jwt_secret);
        match config.secrets.database_url {
            Some(val) => env::set_var("DATABASE_URL", val),
            None => env::set_var("DATABASE_URL", empty),
        }
        match config.secrets.turso_auth_token {
            Some(val) => env::set_var("TURSO_AUTH_TOKEN", val),
            None => env::set_var("TURSO_AUTH_TOKEN", empty),
        }
        match config.secrets_bucket {
            Some(val) => env::set_var("SECRETS_BUCKET", val),
            None => env::set_var("SECRETS_BUCKET", empty),
        }
        match config.secrets_object {
            Some(val) => env::set_var("SECRETS_OBJECT", val),
            None => env::set_var("SECRETS_OBJECT", empty),
        }
        match config.cache_type {
            Some(val) => env::set_var("CACHE_TYPE", val.to_string()),
            None => env::set_var("CACHE_TYPE", empty),
        }
        match config.cache_ttl {
            Some(val) => env::set_var("CACHE_TTL", val.to_string()),
            None => env::set_var("CACHE_TTL", empty),
        }
    }

    fn remove_envars() {
        env::remove_var("SVC_ENDPOINT");
        env::remove_var("SVC_PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("ENVIRONMENT");
        env::remove_var("DATA_SOURCE");
        env::remove_var("JWT_SECRET");
        env::remove_var("DATABASE_URL");
        env::remove_var("TURSO_AUTH_TOKEN");
        env::remove_var("SECRETS_BUCKET");
        env::remove_var("SECRETS_OBJECT");
        env::remove_var("CACHE_TYPE");
        env::remove_var("CACHE_TTL");
    }
}
