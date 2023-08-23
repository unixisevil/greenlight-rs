use tracing_subscriber::{filter::EnvFilter, fmt, fmt::format::FmtSpan, prelude::*};
use anyhow::Context;
use tokio::task::JoinError;
use std::fmt::{Debug, Display};

use greenlight::{Error, Config, build, run_mail_worker};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let fmt_layer = fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_level(true);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_default_env())
        .init();

    let config = Config::new()?;
    tracing::info!("config = {:?}", config);

    let sock_addr: std::net::SocketAddr  = 
        format!("{}:{}", config.addr, config.port)
        .parse()
        .context("not provide valid addr")
        .map_err(Error::UnexpectedError)?;
    

    let (mailer, api_server) =  build(config)?;

    let mail_task =  tokio::spawn(run_mail_worker(mailer));
    let api_task =  tokio::spawn(api_server.run(sock_addr));

    tokio::select! {
        o = api_task  => report_exit("api worker",    Ok(o)),
        o = mail_task =>  report_exit("mail worker", o),
    };
    
    Ok(())
}


fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}


