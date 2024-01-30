use std::env;

/// Struct Config for setup environment variables
#[derive(PartialEq, Debug)]
pub struct Config {
    pub svc_endpoint: String,
    pub svc_port: String,
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
        let postgre_user: String = "admin".to_string();
        let postgre_password: String = "admin-password".to_string();
        let postgre_db: String = "testing".to_string();
        let postgre_host: String = "127.0.0.1".to_string();
        let postgre_port: String = "5432".to_string();

        Self {
            svc_endpoint,
            svc_port,
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
            postgre_user,
            postgre_password,
            postgre_db,
            postgre_host,
            postgre_port,
        }
    }
}
