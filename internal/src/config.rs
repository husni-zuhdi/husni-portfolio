use std::env;
use tracing::{error, warn};

/// Struct Config for setup environment variables
#[derive(PartialEq, Debug, Clone)]
pub struct Config {
    pub svc_endpoint: String,
    pub svc_port: String,
    pub log_level: tracing::Level,
    pub environment: String,
    pub data_source: String,
    pub database_url: String,
    pub turso_auth_token: String,
    pub filesystem_dir: String,
    pub gh_owner: String,
    pub gh_repo: String,
    pub gh_branch: String,
}

impl Default for Config {
    /// By default running on localhost:8080 in release
    /// with log-level info and data from memory
    fn default() -> Self {
        let svc_endpoint: String = "localhost".to_string();
        let svc_port: String = "8080".to_string();
        let log_level = tracing::Level::INFO;
        let environment: String = "release".to_string();
        let data_source: String = "memory".to_string();
        let database_url: String = "".to_string();
        let turso_auth_token: String = "".to_string();
        let filesystem_dir: String = "".to_string();
        let gh_owner: String = "".to_string();
        let gh_repo: String = "".to_string();
        let gh_branch: String = "".to_string();

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
        }
    }
}

impl Config {
    /// Parse optional environment variable to setup the envar and set default
    fn parse_optional_envar(envar: &str, default: &str) -> String {
        match env::var(&envar) {
            Err(e) => {
                warn!(
                    "Failed to load {} environment variable. Set default to '{}'. Error {}",
                    &envar, &default, e
                );
                default.to_string()
            }
            Ok(val) => match val.as_str() {
                "" => default.to_string(),
                _ => val,
            },
        }
    }
    /// from_envar
    /// Setup config from environment variables
    pub fn from_envar() -> Self {
        // Mandatory
        let svc_endpoint: String = env::var("SVC_ENDPOINT")
            .expect("Failed to load SVC_ENDPOINT environment variable. Double check your config");
        let svc_port: String = env::var("SVC_PORT")
            .expect("failed to load SVC_PORT environment variable. Double check your config");

        // Optional
        let log_level: tracing::Level = match env::var("LOG_LEVEL") {
            Err(e) => {
                warn!(
                    "Failed to load LOG_LEVEL environment variable. Set default to 'info'. Error {}", e
                );
                tracing::Level::INFO
            }
            Ok(val) => match val.as_str() {
                "error" => tracing::Level::ERROR,
                "warn" => tracing::Level::WARN,
                "info" => tracing::Level::INFO,
                "debug" => tracing::Level::DEBUG,
                "trace" => tracing::Level::TRACE,
                _ => tracing::Level::INFO,
            },
        };
        let environment: String = Self::parse_optional_envar("ENVIRONMENT", "release");
        // let data_source: String = Self::parse_optional_envar("DATA_SOURCE", "memory");
        let data_source: String = match env::var("DATA_SOURCE") {
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
        };
        let database_url: String = Self::parse_optional_envar("DATABASE_URL", "");
        let turso_auth_token: String = Self::parse_optional_envar("TURSO_AUTH_TOKEN", "");
        let filesystem_dir: String = Self::parse_optional_envar("FILESYSTEM_DIR", "");
        let gh_owner: String = Self::parse_optional_envar("GITHUB_OWNER", "");
        let gh_repo: String = Self::parse_optional_envar("GITHUB_REPO", "");
        let gh_branch: String = Self::parse_optional_envar("GITHUB_BRANCH", "");

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
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let svc_endpoint: String = "localhost".to_string();
        let svc_port: String = "8080".to_string();
        let log_level = tracing::Level::INFO;
        let environment: String = "release".to_string();
        let data_source: String = "memory".to_string();
        let database_url: String = "".to_string();
        let turso_auth_token: String = "".to_string();
        let filesystem_dir: String = "".to_string();
        let gh_owner: String = "".to_string();
        let gh_repo: String = "".to_string();
        let gh_branch: String = "".to_string();

        let result = Config::default();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.data_source, data_source);
        assert_eq!(result.database_url, database_url);
        assert_eq!(result.turso_auth_token, turso_auth_token);
        assert_eq!(result.filesystem_dir, filesystem_dir);
        assert_eq!(result.gh_owner, gh_owner);
        assert_eq!(result.gh_repo, gh_repo);
        assert_eq!(result.gh_branch, gh_branch);
    }

    #[test]
    fn test_from_envar_without_optionals() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let log_level = "";
        let expected_log_level = tracing::Level::INFO;
        let environment = "";
        let expected_environment = "release";
        let data_source = "";
        let expected_data_source = "memory";
        let database_url = "";
        let turso_auth_token = "";
        let filesystem_dir = "";
        let gh_owner = "";
        let gh_repo = "";
        let gh_branch = "";

        set_envars(
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
        );

        let result = Config::from_envar();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, expected_log_level);
        assert_eq!(result.environment, expected_environment);
        assert_eq!(result.data_source, expected_data_source);
        assert_eq!(result.database_url, database_url);
        assert_eq!(result.turso_auth_token, turso_auth_token);
        assert_eq!(result.filesystem_dir, filesystem_dir);
        assert_eq!(result.gh_owner, gh_owner);
        assert_eq!(result.gh_repo, gh_repo);
        assert_eq!(result.gh_branch, gh_branch);

        remove_envars()
    }

    #[test]
    fn test_from_envar_with_optionals() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let log_level = "info";
        let expected_log_level = tracing::Level::INFO;
        let environment = "dev";
        let data_source = "sqlite";
        let database_url = "sqlite:husni-portfolio.db";
        let turso_auth_token = "";
        let filesystem_dir = "";
        let gh_owner = "husni-zuhdi";
        let gh_repo = "husni-blog-resources";
        let gh_branch = "main";

        set_envars(
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
        );

        let result = Config::from_envar();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, expected_log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.data_source, data_source);
        assert_eq!(result.database_url, database_url);
        assert_eq!(result.turso_auth_token, turso_auth_token);
        assert_eq!(result.filesystem_dir, filesystem_dir);
        assert_eq!(result.gh_owner, gh_owner);
        assert_eq!(result.gh_repo, gh_repo);
        assert_eq!(result.gh_branch, gh_branch);

        remove_envars()
    }

    fn set_envars(
        svc_endpoint: &str,
        svc_port: &str,
        log_level: &str,
        environment: &str,
        data_source: &str,
        database_url: &str,
        turso_auth_token: &str,
        filesystem_dir: &str,
        gh_owner: &str,
        gh_repo: &str,
        gh_branch: &str,
    ) {
        env::set_var("SVC_ENDPOINT", svc_endpoint);
        env::set_var("SVC_PORT", svc_port);
        env::set_var("LOG_LEVEL", log_level);
        env::set_var("ENVIRONMENT", environment);
        env::set_var("DATA_SOURCE", data_source);
        env::set_var("DATABASE_URL", database_url);
        env::set_var("TURSO_AUTH_TOKEN", turso_auth_token);
        env::set_var("FILESYSTEM_DIR", filesystem_dir);
        env::set_var("GITHUB_OWNER", gh_owner);
        env::set_var("GITHUB_REPO", gh_repo);
        env::set_var("GITHUB_BRANCH", gh_branch);
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
    }
}
