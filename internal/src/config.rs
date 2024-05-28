use std::env;

/// Struct Config for setup environment variables
#[derive(PartialEq, Debug)]
pub struct Config {
    pub svc_endpoint: String,
    pub svc_port: String,
    pub log_level: String,
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
        let postgre_user: String = "admin".to_string();
        let postgre_password: String = "admin-password".to_string();
        let postgre_db: String = "testing".to_string();
        let postgre_host: String = "127.0.0.1".to_string();
        let postgre_port: String = "5432".to_string();

        Self {
            svc_endpoint,
            svc_port,
            log_level,
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
        let postgre_user: String =
            env::var("POSTGRE_USER").expect("Failed to load POSTGRE_USER environment variable");
        let postgre_password: String = env::var("POSTGRE_PASSWORD")
            .expect("Failed to load POSTGRE_PASSWORD environment variable");
        let postgre_db: String =
            env::var("POSTGRE_DB").expect("Failed to load POSTGRE_DB environment variable");
        let postgre_host: String =
            env::var("POSTGRE_HOST").expect("Failed to load POSTGRE_DB environment variable");
        let postgre_port: String =
            env::var("POSTGRE_PORT").expect("Failed to load POSTGRE_PORT environment variable");

        Self {
            svc_endpoint,
            svc_port,
            log_level,
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
        let postgre_user: String = "admin".to_string();
        let postgre_password: String = "admin-password".to_string();
        let postgre_db: String = "testing".to_string();
        let postgre_host: String = "127.0.0.1".to_string();
        let postgre_port: String = "5432".to_string();

        let result = Config::default();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
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
        let postgre_user = "admin";
        let postgre_password = "admin-password";
        let postgre_db = "testing";
        let postgre_host = "127.0.0.1";
        let postgre_port = "5432";

        env::set_var("SVC_ENDPOINT", svc_endpoint);
        env::set_var("SVC_PORT", svc_port);
        env::set_var("LOG_LEVEL", log_level);
        env::set_var("POSTGRE_USER", postgre_user);
        env::set_var("POSTGRE_PASSWORD", postgre_password);
        env::set_var("POSTGRE_DB", postgre_db);
        env::set_var("POSTGRE_HOST", postgre_host);
        env::set_var("POSTGRE_PORT", postgre_port);

        let result = Config::from_envar();
        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.postgre_user, postgre_user);
        assert_eq!(result.postgre_password, postgre_password);
        assert_eq!(result.postgre_db, postgre_db);
        assert_eq!(result.postgre_host, postgre_host);
        assert_eq!(result.postgre_port, postgre_port);
    }
}
