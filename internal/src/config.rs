use std::env;
use tracing::{error, warn};

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
    /// JWT Secret.
    /// Secret to encode JWT in authenticated-pages.
    /// Default to `secret` but highly advised to provide your own value.
    pub jwt_secret: String,
    /// Database URL (Optional)
    /// Database URL (or Path). **Required** if you use `sqlite` or `turso`
    /// as DATA_SOURCE.
    /// Example:
    ///     - sqlite:husni-portfolio.db
    ///     - libsql://husni-portfolio.asia.turso.io
    /// Default to None.
    pub database_url: Option<String>,
    /// Turso Auth Token (Optional)
    /// Authentication token for turso database. **Required** if you use
    /// `turso` as DATA_SOURCE..
    /// Default to None
    pub turso_auth_token: Option<String>,
    /// Filesystem Dir (Optional; Deprecated)
    /// Directory of blog markdown files.
    /// Default to None.
    pub filesystem_dir: Option<String>,
    /// Github Owner (Optional; Deprecated)
    /// Github owner of blogs repository.
    /// Default to None.
    pub gh_owner: Option<String>,
    /// Github Repository (Optional; Deprecated)
    /// Github repository name.
    /// Default to None.
    pub gh_repo: Option<String>,
    /// Github Branch (Optional; Deprecated)
    /// Branch of blog github repository.
    /// Default to None.
    pub gh_branch: Option<String>,
    /// Google Cloud Storage (GCS) Bukcet Name (Optional)
    /// GCS bucket name.
    /// Default to None.
    pub bucket_name: Option<String>,
    /// Google Cloud Storage (GCS) Secret URI (Optional)
    /// GCS URI of secret file. An alternative to load secretive envars
    /// other than Google Secret Manager.
    /// If set, will override config secrets.
    /// List of secrets:
    /// - JWT_SECRET
    /// - DATABASE_URL
    /// - TURSO_AUTH_TOKEN
    /// - BUCKET_NAME
    ///
    /// Example: gs://husni-portfolio-bucket/secret/secret
    /// Default to None.
    pub secret_uri: Option<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Environment {
    Development,
    Release,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
            jwt_secret,
            database_url: None,
            turso_auth_token: None,
            filesystem_dir: None,
            gh_owner: None,
            gh_repo: None,
            gh_branch: None,
            bucket_name: None,
            secret_uri: None,
        }
    }
}

impl Config {
    /// Setup config from environment variables
    pub fn from_envar() -> Self {
        // Required
        let svc_endpoint: String = env::var("SVC_ENDPOINT")
            .expect("Failed to load SVC_ENDPOINT environment variable. Double check your config");
        let svc_port: String = env::var("SVC_PORT")
            .expect("failed to load SVC_PORT environment variable. Double check your config");
        let log_level = Self::parse_log_level();
        let environment = Self::parse_environment();
        let data_source = Self::parse_data_source();
        let mut jwt_secret: String = env::var("JWT_SECRET")
            .expect("failed to load JWT_SECRET environment variable. Double check your config");

        // Optional
        let mut database_url = Self::parse_optional("DATABASE_URL");
        let mut turso_auth_token = Self::parse_optional("TURSO_AUTH_TOKEN");
        let filesystem_dir = Self::parse_optional("FILESYSTEM_DIR");
        let gh_owner = Self::parse_optional("GITHUB_OWNER");
        let gh_repo = Self::parse_optional("GITHUB_REPO");
        let gh_branch = Self::parse_optional("GITHUB_BRANCH");
        let mut bucket_name = Self::parse_optional("BUCKET_NAME");
        let secret_uri = Self::parse_optional("SECRET_URI");

        // TODO
        // Check SECRET_URI
        // If set, try to load the secret then override all secrets.

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
            data_source,
            database_url,
            turso_auth_token,
            filesystem_dir,
            gh_owner,
            gh_repo,
            gh_branch,
            jwt_secret,
            bucket_name,
            secret_uri,
        }
    }
    /// Parse Optional environment variables
    fn parse_optional(envar: &str) -> Option<String> {
        match env::var(envar) {
            Err(e) => {
                warn!(
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
                warn!(
                "Failed to load ENVIRONMENT environment variable. Set default to 'Release'. Error {}",
                e
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
                warn!(
                "Failed to load LOG_LEVEL environment variable. Set default to 'info'. Error {}",
                e
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
                warn!(
                "Failed to load DATA_SOURCE environment variable. Set default to 'memory'. Error {}", e
                );
                "memory".to_string()
            }
            Ok(val) => match val.as_str() {
                "memory" | "sqlite" | "turso" => val,
                _ => {
                    error!(
                        "Data Source type {} is not supported! Default to 'memory'.",
                        val
                    );
                    "memory".to_string()
                }
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
        assert_eq!(result.jwt_secret, jwt_secret);
        assert_eq!(result.database_url, None);
        assert_eq!(result.turso_auth_token, None);
        assert_eq!(result.filesystem_dir, None);
        assert_eq!(result.gh_owner, None);
        assert_eq!(result.gh_repo, None);
        assert_eq!(result.gh_branch, None);
        assert_eq!(result.bucket_name, None);
    }

    #[test]
    fn test_from_envar_without_optionals() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let log_level = tracing::Level::INFO;
        let expected_log_level = tracing::Level::INFO;
        let environment = Environment::Release;
        let expected_environment = Environment::Release;
        let data_source = "";
        let expected_data_source = "memory";
        let jwt_secret = "secret";
        let empty = Some("".to_string());

        set_envars(Config {
            svc_endpoint: svc_endpoint.to_string(),
            svc_port: svc_port.to_string(),
            log_level,
            environment,
            data_source: data_source.to_string(),
            jwt_secret: jwt_secret.to_string(),
            database_url: empty.clone(),
            turso_auth_token: empty.clone(),
            filesystem_dir: empty.clone(),
            gh_owner: empty.clone(),
            gh_repo: empty.clone(),
            gh_branch: empty.clone(),
            bucket_name: empty.clone(),
            secret_uri: empty,
        });

        let result = Config::from_envar();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, expected_log_level);
        assert_eq!(result.environment, expected_environment);
        assert_eq!(result.data_source, expected_data_source);
        assert_eq!(result.jwt_secret, jwt_secret);
        assert_eq!(result.database_url, None);
        assert_eq!(result.turso_auth_token, None);
        assert_eq!(result.filesystem_dir, None);
        assert_eq!(result.gh_owner, None);
        assert_eq!(result.gh_repo, None);
        assert_eq!(result.gh_branch, None);
        assert_eq!(result.bucket_name, None);
        assert_eq!(result.secret_uri, None);

        remove_envars()
    }

    #[test]
    fn test_from_envar_with_optionals() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let expected_log_level = tracing::Level::INFO;
        let environment = Environment::Development;
        let data_source = "sqlite";
        let jwt_secret = "secret";
        let database_url = Some("sqlite:husni-portfolio.db".to_string());
        let turso_auth_token = Some("turso_token_123456".to_string());
        let filesystem_dir = Some("/tmp/blogs".to_string());
        let gh_owner = Some("husni-zuhdi".to_string());
        let gh_repo = Some("husni-blog-resources".to_string());
        let gh_branch = Some("main".to_string());
        let bucket_name = Some("".to_string());
        let secret_uri = Some("".to_string());

        set_envars(Config {
            svc_endpoint: svc_endpoint.to_string(),
            svc_port: svc_port.to_string(),
            log_level: tracing::Level::INFO,
            environment: Environment::Development,
            data_source: data_source.to_string(),
            jwt_secret: jwt_secret.to_string(),
            database_url: database_url.clone(),
            turso_auth_token: turso_auth_token.clone(),
            filesystem_dir: filesystem_dir.clone(),
            gh_owner: gh_owner.clone(),
            gh_repo: gh_repo.clone(),
            gh_branch: gh_branch.clone(),
            bucket_name,
            secret_uri,
        });

        let result = Config::from_envar();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, expected_log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.data_source, data_source);
        assert_eq!(result.jwt_secret, jwt_secret);
        assert_eq!(result.database_url, database_url);
        assert_eq!(result.turso_auth_token, turso_auth_token);
        assert_eq!(result.filesystem_dir, filesystem_dir);
        assert_eq!(result.gh_owner, gh_owner);
        assert_eq!(result.gh_repo, gh_repo);
        assert_eq!(result.gh_branch, gh_branch);
        assert_eq!(result.bucket_name, None);
        assert_eq!(result.secret_uri, None);

        remove_envars()
    }

    fn set_envars(config: Config) {
        env::set_var("SVC_ENDPOINT", config.svc_endpoint);
        env::set_var("SVC_PORT", config.svc_port);
        env::set_var("LOG_LEVEL", config.log_level.to_string());
        env::set_var("ENVIRONMENT", config.environment.to_string());
        env::set_var("DATA_SOURCE", config.data_source);
        env::set_var("JWT_SECRET", config.jwt_secret);
        env::set_var("DATABASE_URL", config.database_url.unwrap());
        env::set_var("TURSO_AUTH_TOKEN", config.turso_auth_token.unwrap());
        env::set_var("FILESYSTEM_DIR", config.filesystem_dir.unwrap());
        env::set_var("GITHUB_OWNER", config.gh_owner.unwrap());
        env::set_var("GITHUB_REPO", config.gh_repo.unwrap());
        env::set_var("GITHUB_BRANCH", config.gh_branch.unwrap());
        env::set_var("BUCKET_NAME", config.bucket_name.unwrap());
        env::set_var("SECRET_URI", config.secret_uri.unwrap());
    }

    fn remove_envars() {
        env::remove_var("SVC_ENDPOINT");
        env::remove_var("SVC_PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("ENVIRONMENT");
        env::remove_var("DATA_SOURCE");
        env::remove_var("DATABASE_URL");
        env::remove_var("TURSO_AUTH_TOKEN");
        env::remove_var("FILESYSTEM_DIR");
        env::remove_var("GITHUB_OWNER");
        env::remove_var("GITHUB_REPO");
        env::remove_var("GITHUB_BRANCH");
        env::remove_var("JWT_SECRET");
        env::remove_var("BUCKET_NAME");
        env::remove_var("SECRET_URI");
    }
}
