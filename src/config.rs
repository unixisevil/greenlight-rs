use std::time::Duration;
use std::num::ParseIntError;
use clap::Parser;

use crate::errors::Error;

#[derive(Parser, PartialEq, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    #[clap(short, long, default_value = "warn")]
    pub log_level: String,
    
    #[clap(short, long, default_value = "0.0.0.0")]
    pub addr: String,

    #[clap(short, long, default_value = "8000")]
    pub port: u16,

    #[clap(long, default_value = "redis://127.0.0.1:6379")]
    pub redis_url: String,

    #[command(flatten)]
    pub pg: DbConfig,

    #[command(flatten)]
    pub mail: MailConfig,
}

#[derive(clap::Args, PartialEq, Debug)]
pub struct DbConfig {
    #[clap(long, default_value = "postgres://green:greenpass@localhost:5432/greenlight?sslmode=disable")]
    pub db_dsn: String,

    #[clap(long, default_value = "8")]
    pub db_max_conn: u32,

    #[arg(value_parser = |arg: &str| -> Result<Duration, ParseIntError> {Ok(Duration::from_secs(arg.parse()?))})]
    #[clap(long, default_value = "2")]
    pub db_connect_timeout: Duration,
}

#[derive(clap::Args, PartialEq, Debug, Clone)]
pub struct MailConfig {
    #[clap(long, default_value = "from@example.com")]
    pub mail_sender: String,

    #[clap(long, default_value = "sandbox.smtp.mailtrap.io")]
    pub mail_host: String, 

    #[clap(long, default_value = "2525")]
    pub mail_port: u16, 

    #[clap(long, default_value = "db8ad43072bf5f")]
    pub mail_username: String,

    #[clap(long, default_value = "234fb598e8fa21")]
    pub mail_password: String,
}


impl Config {
    pub fn new() -> Result<Config, Error> {
        let config = Config::parse();
                
        let port = std::env::var("GREENLIGHT_PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(Error::ConfigParse)?;

        let addr = std::env::var("GREENLIGHT_ADDR")
            .ok()
            .unwrap_or(config.addr);


        let redis_url = std::env::var("GREENLIGHT_REDIS_URL")
            .ok()
            .unwrap_or(config.redis_url);


        let db_dsn = std::env::var("GREENLIGHT_DB_DSN")
            .ok()
            .unwrap_or(config.pg.db_dsn);

        let max_conn = std::env::var("GREENLIGHT_DB_MAXCONN")
            .ok()
            .map(|val| val.parse::<u32>())
            .unwrap_or(Ok(config.pg.db_max_conn))
            .map_err(Error::ConfigParse)?;

        let connect_timeout = std::env::var("GREENLIGHT_DB_TIMEOUT")
            .ok()
            .map(|val| {
                   let n = val.parse::<u64>().map_err(Error::ConfigParse)?; 
                   Ok::<Duration, Error>(Duration::from_secs(n))
               }
            )
            .unwrap_or(Ok(config.pg.db_connect_timeout))
            .unwrap();

        let mail_sender = std::env::var("GREENLIGHT_MAIL_SENDER")
            .ok()
            .unwrap_or(config.mail.mail_sender);

        let mail_host = std::env::var("GREENLIGHT_MAIL_HOST")
            .ok()
            .unwrap_or(config.mail.mail_host);

        let mail_port = std::env::var("GREENLIGHT_MAIL_PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.mail.mail_port))
            .map_err(Error::ConfigParse)?;

        let mail_username = std::env::var("GREENLIGHT_MAIL_USERNAME")
            .ok()
            .unwrap_or(config.mail.mail_username);

        let mail_password = std::env::var("GREENLIGHT_MAIL_PASSWORD")
            .ok()
            .unwrap_or(config.mail.mail_password);
            
        Ok(Config {
            log_level: config.log_level,
            addr,
            port,  
            redis_url,
            pg : DbConfig { db_dsn,  db_max_conn: max_conn, db_connect_timeout: connect_timeout },
            mail: MailConfig { mail_sender, mail_host, mail_port, mail_username, mail_password }
        })
    }
}

