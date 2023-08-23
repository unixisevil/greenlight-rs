use anyhow::Context;
use askama::Template;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use redis::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::instrument;

use crate::config::MailConfig;

#[derive(Clone)]
pub struct Mailer {
    sender_addr: String,
    builder:  SmtpClientBuilder<String>,
    redis:    Client,
}

impl Mailer {
    pub fn new(config: MailConfig, redis:  Client) -> Self {
        let builder = SmtpClientBuilder::new(config.mail_host, config.mail_port)
            .implicit_tls(false)
            .credentials((config.mail_username, config.mail_password));

        Self {
            sender_addr: config.mail_sender,
            builder, 
            redis,
        }
    }
    
    async fn try_pop_send(&self) -> Result<Outcome, anyhow::Error> {
        let mut conn = self.redis
            .get_async_connection()
            .await
            .context("failed to get redis conn in pop task")?;

        let ret: Option<Vec<u8>> = redis::cmd("RPOP")
            .arg("mail_queue")
            .query_async(&mut conn)
            .await
            .context("pop mail task failed")?;

        if ret.is_none() {
            return Ok(Outcome::EmptyQueue);
        }

        let task: MailTask =
            bincode::deserialize(&ret.unwrap()).context("failed to deserialize mail task")?;

        if let Err(e) = self.send(&task).await {
            tracing::debug!(err=%e, "failed to executing  mail task");
            return push_task(&self.redis, &task).await.map(|_| Ok(Outcome::TaskCompleted))?;
        }

        Ok(Outcome::TaskCompleted)
    }

    async fn send(&self, task: &MailTask) -> Result<(), mail_send::Error> {
        let message = MessageBuilder::new()
            .from(("", self.sender_addr.as_str()))
            .to(task.recipient.as_str())
            .subject(task.subject.as_str())
            .html_body(task.html_body.as_str())
            .text_body(task.plain_body.as_str());

            self.builder.connect()
            .await?
            .send(message)
            .await?;

        Ok(())
    }
}

pub enum Outcome {
    TaskCompleted,
    EmptyQueue,
}

#[instrument(skip_all)]
async fn worker_loop(mailer: Mailer) -> Result<(), anyhow::Error> {
        loop {
            match mailer.try_pop_send().await {
                Ok(Outcome::EmptyQueue) => {
                    tracing::debug!("get EmptyQueue");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    tracing::error!(err = %e, "try_pop_send");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Ok(Outcome::TaskCompleted) => {
                    tracing::info!("mail task ok");
                }
            }
        }
}

pub async fn run_mail_worker(mailer: Mailer) -> Result<(), anyhow::Error> {
      worker_loop(mailer).await
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MailTask {
    recipient: String,
    subject: String,
    html_body: String,
    plain_body: String,
}

pub async fn push_task(client: &Client, task: &MailTask) -> Result<(), anyhow::Error> {
    let mut conn = client
        .get_async_connection()
        .await
        .context("failed to get redis conn in push_task")?;

    let task_msg = bincode::serialize(&task).context("failed to serialize task with bincode")?;

    redis::cmd("LPUSH")
        .arg("mail_queue")
        .arg(task_msg)
        .query_async(&mut conn)
        .await
        .context("push mail task failed")?;

    Ok(())
}

#[derive(Copy, Clone, Debug, Default)]
enum MailPart {
    #[default]
    Subject,
    HtmlBody,
    PlainBody,
}

trait MutablePart: askama::Template + Sized {
    fn part(&mut self) -> &mut MailPart;

    fn gen_task(mut self, recipient: String) -> Result<MailTask, askama::Error> {
        *self.part() = MailPart::Subject;
        let subject = self.render()?;
        *self.part() = MailPart::HtmlBody;
        let html_body = self.render()?;
        *self.part() = MailPart::PlainBody;
        let plain_body = self.render()?;

        Ok(MailTask {
            recipient,
            subject,
            html_body,
            plain_body,
        })
    }
}

#[derive(Template, Default)]
#[template(path = "user_welcome.tmpl", escape = "html")]
pub struct Welcome {
    part: MailPart,
    user_id: i64,
    activation_token: String,
}

impl MutablePart for Welcome {
    fn part(&mut self) -> &mut MailPart {
        &mut self.part
    }
}

impl Welcome {
    pub fn new(user_id: i64, activation_token: String) -> Self {
        Self {
            part:  MailPart::default(),
            user_id,
            activation_token,
        }
    }

    pub fn gen_task(self, recipient: String) -> Result<MailTask, askama::Error> {
        <Self as MutablePart>::gen_task(self, recipient)
    }
}

#[derive(Template, Default)]
#[template(path = "password_reset.tmpl", escape = "html")]
pub struct PasswordReset {
    part: MailPart,
    reset_token: String,
}

impl MutablePart for PasswordReset {
    fn part(&mut self) -> &mut MailPart {
        &mut self.part
    }
}

impl PasswordReset {
    pub fn new(reset_token: String) -> Self {
        Self {
            part: MailPart::default(),
            reset_token,
        }
    }

    pub fn gen_task(self, recipient: String) -> Result<MailTask, askama::Error> {
        <Self as MutablePart>::gen_task(self, recipient)
    }
}

#[derive(Template, Default)]
#[template(path = "token_activation.tmpl", escape = "html")]
pub struct TokenActivation {
    part: MailPart,
    activation_token: String,
}

impl MutablePart for TokenActivation {
    fn part(&mut self) -> &mut MailPart {
        &mut self.part
    }
}

impl TokenActivation {
    pub fn new(activation_token: String) -> Self {
        Self{
            part: MailPart::default(),
            activation_token,
        }
    }

    pub fn gen_task(self, recipient: String) -> Result<MailTask, askama::Error> {
        <Self as MutablePart>::gen_task(self, recipient)
    }
}

