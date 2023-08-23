use tokio::task::JoinHandle;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, Secret};
use anyhow::Context;
use crate::Error;

pub (super) async fn gen_passwordhash(password: Secret<String>)  -> Result<Secret<String>, Error> {
    let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await
        .context("Failed spawn hash task")?
        .context("Failed to hash password")?;
      
     Ok(password_hash)
}

pub (super) async fn verify_passwordhash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), Error> {
    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, password_candidate)
    })
    .await
    .context("Failed to spawn verify task.")??;

    Ok(())
}


fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}


fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();

    Ok(Secret::new(password_hash))
}

fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), Error> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.").map_err(Error::UnexpectedError)?;

    Argon2::default()
    .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
    )
    .map_err(|_| Error::InvalidCredentials)?;

    Ok(())
}
