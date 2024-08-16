use log::warn;
use std::env;

/// Struct Config for setup environment variables
#[derive(PartialEq, Debug, Clone)]
pub struct Config {
    pub svc_endpoint: String,
    pub svc_port: String,
    pub log_level: String,
    pub environment: String,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub postgres_host: String,
    pub postgres_port: String,
    pub gh_owner: String,
    pub gh_repo: String,
    pub gh_branch: String,
}

impl Default for Config {
    fn default() -> Self {
        let svc_endpoint: String = "127.0.0.1".to_string();
        let svc_port: String = "8080".to_string();
        let log_level: String = "info".to_string();
        let environment: String = "prod".to_string();
        let postgres_user: String = "".to_string();
        let postgres_password: String = "".to_string();
        let postgres_db: String = "".to_string();
        let postgres_host: String = "".to_string();
        let postgres_port: String = "".to_string();
        let gh_owner: String = "".to_string();
        let gh_repo: String = "".to_string();
        let gh_branch: String = "".to_string();

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
            postgres_user,
            postgres_password,
            postgres_db,
            postgres_host,
            postgres_port,
            gh_owner,
            gh_repo,
            gh_branch,
        }
    }
}

impl Config {
    /// from_envar
    /// Setup config from environment variables
    pub fn from_envar() -> Self {
        // Mandatory
        let svc_endpoint: String = env::var("SVC_ENDPOINT")
            .expect("Failed to load SVC_ENDPOINT environment variable. Double check your config");
        let svc_port: String = env::var("SVC_PORT")
            .expect("failed to load SVC_PORT environment variable. Double check your config");

        // Optional
        let log_level: String = env::var("LOG_LEVEL").unwrap_or_else(|_| {
            warn!("Failed to load LOG_LEVEL environment variable. Set default to 'info'");
            "info".to_string()
        });
        let environment: String = env::var("ENVIRONMENT").unwrap_or_else(|_| {
            warn!("Failed to load ENVIRONMENT environment variable. Set default to 'prod'");
            "prod".to_string()
        });
        let postgres_user: String = env::var("POSTGRES_USER").unwrap_or_else(|_| {
            warn!("Failed to load POSTGRES_USER environment variable. Set default to ''");
            "".to_string()
        });
        let postgres_password: String = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| {
            warn!("Failed to load POSTGRES_PASSWORD environment variable. Set default to ''");
            "".to_string()
        });
        let postgres_db: String = env::var("POSTGRES_DB").unwrap_or_else(|_| {
            warn!("Failed to load POSTGRES_DB environment variable. Set default to ''");
            "".to_string()
        });
        let postgres_host: String = env::var("POSTGRES_HOST").unwrap_or_else(|_| {
            warn!("Failed to load POSTGRES_HOST environment variable. Set default to ''");
            "".to_string()
        });
        let postgres_port: String = env::var("POSTGRES_PORT").unwrap_or_else(|_| {
            warn!("Failed to load POSTGRES_PORT environment variable. Set default to ''");
            "".to_string()
        });
        let gh_owner: String = env::var("GITHUB_OWNER").unwrap_or_else(|_| {
            warn!("Failed to load GITHUB_OWNER environment variable. Set default to ''");
            "".to_string()
        });
        let gh_repo: String = env::var("GITHUB_REPO").unwrap_or_else(|_| {
            warn!("Failed to load GITHUB_REPO environment variable. Set default to ''");
            "".to_string()
        });
        let gh_branch: String = env::var("GITHUB_BRANCH").unwrap_or_else(|_| {
            warn!("Failed to load GITHUB_BRANCH environment variable. Set default to ''");
            "".to_string()
        });

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
            postgres_user,
            postgres_password,
            postgres_db,
            postgres_host,
            postgres_port,
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
        let svc_endpoint: String = "127.0.0.1".to_string();
        let svc_port: String = "8080".to_string();
        let log_level: String = "info".to_string();
        let environment: String = "prod".to_string();
        let postgres_user: String = "".to_string();
        let postgres_password: String = "".to_string();
        let postgres_db: String = "".to_string();
        let postgres_host: String = "".to_string();
        let postgres_port: String = "".to_string();
        let gh_owner: String = "".to_string();
        let gh_repo: String = "".to_string();
        let gh_branch: String = "".to_string();

        let result = Config::default();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.postgres_user, postgres_user);
        assert_eq!(result.postgres_password, postgres_password);
        assert_eq!(result.postgres_db, postgres_db);
        assert_eq!(result.postgres_host, postgres_host);
        assert_eq!(result.postgres_port, postgres_port);
        assert_eq!(result.gh_owner, gh_owner);
        assert_eq!(result.gh_repo, gh_repo);
        assert_eq!(result.gh_branch, gh_branch);
    }

    #[test]
    fn test_from_envar_without_optionals() {
        let svc_endpoint = "127.0.0.1";
        let svc_port = "8080";
        let log_level = "info";
        let environment = "dev";
        let postgres_user = "";
        let postgres_password = "";
        let postgres_db = "";
        let postgres_host = "";
        let postgres_port = "";
        let gh_owner = "";
        let gh_repo = "";
        let gh_branch = "";

        env::set_var("SVC_ENDPOINT", svc_endpoint);
        env::set_var("SVC_PORT", svc_port);
        env::set_var("LOG_LEVEL", log_level);
        env::set_var("ENVIRONMENT", environment);
        env::set_var("POSTGRES_USER", postgres_user);
        env::set_var("POSTGRES_PASSWORD", postgres_password);
        env::set_var("POSTGRES_DB", postgres_db);
        env::set_var("POSTGRES_HOST", postgres_host);
        env::set_var("POSTGRES_PORT", postgres_port);
        env::set_var("GITHUB_OWNER", gh_owner);
        env::set_var("GITHUB_REPO", gh_repo);
        env::set_var("GITHUB_BRANCH", gh_branch);

        let result = Config::from_envar();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.postgres_user, postgres_user);
        assert_eq!(result.postgres_password, postgres_password);
        assert_eq!(result.postgres_db, postgres_db);
        assert_eq!(result.postgres_host, postgres_host);
        assert_eq!(result.postgres_port, postgres_port);
        assert_eq!(result.gh_owner, gh_owner);
        assert_eq!(result.gh_repo, gh_repo);
        assert_eq!(result.gh_branch, gh_branch);
    }

    #[test]
    fn test_from_envar_with_optionals() {
        let svc_endpoint = "127.0.0.1";
        let svc_port = "8080";
        let log_level = "info";
        let environment = "dev";
        let postgres_user = "admin";
        let postgres_password = "admin-password";
        let postgres_db = "testing";
        let postgres_host = "127.0.0.1";
        let postgres_port = "5432";
        let gh_owner = "husni-zuhdi";
        let gh_repo = "husni-blog-resources";
        let gh_branch = "main";

        env::set_var("SVC_ENDPOINT", svc_endpoint);
        env::set_var("SVC_PORT", svc_port);
        env::set_var("LOG_LEVEL", log_level);
        env::set_var("ENVIRONMENT", environment);
        env::set_var("POSTGRES_USER", postgres_user);
        env::set_var("POSTGRES_PASSWORD", postgres_password);
        env::set_var("POSTGRES_DB", postgres_db);
        env::set_var("POSTGRES_HOST", postgres_host);
        env::set_var("POSTGRES_PORT", postgres_port);
        env::set_var("GITHUB_OWNER", gh_owner);
        env::set_var("GITHUB_REPO", gh_repo);
        env::set_var("GITHUB_BRANCH", gh_branch);

        let result = Config::from_envar();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.postgres_user, postgres_user);
        assert_eq!(result.postgres_password, postgres_password);
        assert_eq!(result.postgres_db, postgres_db);
        assert_eq!(result.postgres_host, postgres_host);
        assert_eq!(result.postgres_port, postgres_port);
        assert_eq!(result.gh_owner, gh_owner);
        assert_eq!(result.gh_repo, gh_repo);
        assert_eq!(result.gh_branch, gh_branch);
    }
}
