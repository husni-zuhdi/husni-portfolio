use std::env;

/// Struct Config for setup environment variables
#[derive(PartialEq, Debug, Clone)]
pub struct Config {
    pub svc_endpoint: String,
    pub svc_port: String,
    pub log_level: String,
    pub environment: String,
    pub postgre_user: String,
    pub postgre_password: String,
    pub postgre_db: String,
    pub postgre_host: String,
    pub postgre_port: String,
}

impl Default for Config {
    fn default() -> Self {
        let svc_endpoint: String = "127.0.0.1".to_string();
        let svc_port: String = "8080".to_string();
        let log_level: String = "info".to_string();
        let environment: String = "dev".to_string();
        let postgre_user: String = "admin".to_string();
        let postgre_password: String = "admin-password".to_string();
        let postgre_db: String = "testing".to_string();
        let postgre_host: String = "127.0.0.1".to_string();
        let postgre_port: String = "5432".to_string();

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
            postgre_user,
            postgre_password,
            postgre_db,
            postgre_host,
            postgre_port,
        }
    }
}

impl Config {
    pub fn from_envar() -> Self {
        let svc_endpoint: String =
            env::var("SVC_ENDPOINT").expect("Failed to load SVC_ENDPOINT environment variable");
        let svc_port: String =
            env::var("SVC_PORT").expect("Failed to load SVC_PORT environment variable");
        let log_level: String =
            env::var("LOG_LEVEL").expect("Failed to load LOG_LEVEL environment variable");
        let environment: String =
            env::var("ENVIRONMENT").expect("Failed to load ENVIRONMENT environment variable");
        let postgre_user: String =
            env::var("POSTGRES_USER").expect("Failed to load POSTGRES_USER environment variable");
        let postgre_password: String = env::var("POSTGRES_PASSWORD")
            .expect("Failed to load POSTGRES_PASSWORD environment variable");
        let postgre_db: String =
            env::var("POSTGRES_DB").expect("Failed to load POSTGRES_DB environment variable");
        let postgre_host: String =
            env::var("POSTGRES_HOST").expect("Failed to load POSTGRES_DB environment variable");
        let postgre_port: String =
            env::var("POSTGRES_PORT").expect("Failed to load POSTGRES_PORT environment variable");

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
            postgre_user,
            postgre_password,
            postgre_db,
            postgre_host,
            postgre_port,
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
        let environment: String = "dev".to_string();
        let postgre_user: String = "admin".to_string();
        let postgre_password: String = "admin-password".to_string();
        let postgre_db: String = "testing".to_string();
        let postgre_host: String = "127.0.0.1".to_string();
        let postgre_port: String = "5432".to_string();

        let result = Config::default();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.postgre_user, postgre_user);
        assert_eq!(result.postgre_password, postgre_password);
        assert_eq!(result.postgre_db, postgre_db);
        assert_eq!(result.postgre_host, postgre_host);
        assert_eq!(result.postgre_port, postgre_port);
    }

    #[test]
    fn test_from_envar() {
        let svc_endpoint = "127.0.0.1";
        let svc_port = "8080";
        let log_level = "info";
        let environment = "dev";
        let postgre_user = "admin";
        let postgre_password = "admin-password";
        let postgre_db = "testing";
        let postgre_host = "127.0.0.1";
        let postgre_port = "5432";

        env::set_var("SVC_ENDPOINT", svc_endpoint);
        env::set_var("SVC_PORT", svc_port);
        env::set_var("LOG_LEVEL", log_level);
        env::set_var("ENVIRONMENT", environment);
        env::set_var("POSTGRES_USER", postgre_user);
        env::set_var("POSTGRES_PASSWORD", postgre_password);
        env::set_var("POSTGRES_DB", postgre_db);
        env::set_var("POSTGRES_HOST", postgre_host);
        env::set_var("POSTGRES_PORT", postgre_port);

        let result = Config::from_envar();
        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
        assert_eq!(result.postgre_user, postgre_user);
        assert_eq!(result.postgre_password, postgre_password);
        assert_eq!(result.postgre_db, postgre_db);
        assert_eq!(result.postgre_host, postgre_host);
        assert_eq!(result.postgre_port, postgre_port);
    }
}
