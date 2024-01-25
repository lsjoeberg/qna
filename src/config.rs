use clap::Parser;
use std::env;

/// Q&A web service API
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Log level (info, warn, or error)
    #[clap(short, long, default_value = "warn")]
    pub log_level: String,
    /// Web server port
    #[clap(short, long, default_value_t = 7878u16)]
    pub port: u16,
    /// Database user
    #[clap(long, default_value = "username")]
    pub db_user: String,
    /// Database password
    #[clap(long)]
    pub db_password: String,
    /// URL for the postgres database
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    /// Port number for the database connection
    #[clap(long, default_value_t = 5432u16)]
    pub db_port: u16,
    /// Database name
    #[clap(long, default_value = "rwd")]
    pub db_name: String,
}

impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        dotenv::dotenv().ok();
        let config = Config::parse();

        // Preflight check that env vars required during runtime are present.
        if env::var("BADWORDS_API_KEY").is_err() {
            panic!("BADWORDS_API_KEY not set");
        }
        if env::var("PASETO_KEY").is_err() {
            panic!("PASETO_KEY not set");
        }

        let port = env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(7878))
            .map_err(handle_errors::Error::ParseError)?;

        let db_user = env::var("POSTGRES_USER").unwrap_or(config.db_user.to_owned());
        let db_password = env::var("POSTGRES_PASSWORD").unwrap_or(config.db_password.to_owned());
        let db_host = env::var("POSTGRES_HOST").unwrap_or(config.db_host.to_owned());
        let db_port = match env::var("POSTGRES_PORT") {
            Ok(s) => s.parse::<u16>().map_err(handle_errors::Error::ParseError)?,
            Err(_) => config.db_port,
        };
        let db_name = env::var("POSTBRES_DB").unwrap_or(config.db_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port,
            db_name,
        })
    }
}