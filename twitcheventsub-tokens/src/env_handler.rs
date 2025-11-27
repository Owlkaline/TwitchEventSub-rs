use std::{fs::OpenOptions, io::Write};

use env_file_reader::read_file;
use log::error;

use crate::{builder::TokenBuilderError, TokenHandler};

pub struct EnvHandler;

impl EnvHandler {
  pub fn load_env(token: &mut TokenHandler) -> Result<(), TokenBuilderError> {
    dbg!(&token.env);
    match read_file(&token.env) {
      Ok(vars) => {
        let client_id = vars.get("CLIENT_ID").map(String::clone);
        let client_secret = vars.get("CLIENT_SECRET").map(String::clone);
        let redirect_url = vars.get("REDIRECT_URL").map(String::clone);

        if client_id.is_none() {
          return Err(TokenBuilderError::ClientIdNotSet);
        }
        if client_secret.is_none() {
          return Err(TokenBuilderError::ClientSecretNotSet);
        }
        if redirect_url.is_none() {
          return Err(TokenBuilderError::RedirectUrlIncorrect);
        }

        token.client_id = client_id.unwrap();
        token.client_secret = client_secret.unwrap();
        token.redirect_url = redirect_url.unwrap();

        Ok(())
      }
      Err(_) => Err(TokenBuilderError::EnvDoesntExist),
    }
  }

  pub fn load_user_token_env(token: &mut TokenHandler) -> bool {
    match read_file(&token.user_token_env) {
      Ok(vars) => {
        let user_token = &vars["UserToken"];
        token.user_token = user_token.to_owned();

        true
      }
      Err(_) => {
        println!("No env file called {:?}", token.user_token_env);
        error!("No env file called {:?}", token.user_token_env);
        false
      }
    }
  }

  pub fn load_refresh_token_env(token: &mut TokenHandler) -> bool {
    match read_file(&token.refresh_token_env) {
      Ok(vars) => {
        let refresh_token = &vars["RefreshToken"];
        token.refresh_token = refresh_token.to_owned();
        true
      }
      Err(_) => {
        error!("No env file called {:?}", token.refresh_token_env);
        false
      }
    }
  }

  pub fn save_env(
    env_file: &str,
    client_id: &str,
    client_secret: &str,
    client_twitch_id: &str,
    redirect_url: &str,
  ) {
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .append(false)
      .write(true)
      .open(env_file)
    {
      let data = format!(
        "CLIENT_ID={}\nCLIENT_SECRET={}\nCLIENT_TWITCH_ID={}\nREDIRECT_URL={}\n",
        client_id, client_secret, client_twitch_id, redirect_url
      );

      if let Err(e) = file.write_all(data.as_bytes()) {
        panic!("{}", e);
      }
      let _ = file.flush();
    }
  }

  pub fn save_user_token(env_file: &str, user_token: &str) {
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .append(false)
      .write(true)
      .open(env_file)
    {
      let data = format!("UserToken={}\n", user_token);

      if let Err(e) = file.write_all(data.as_bytes()) {
        panic!("{}", e);
      }
      let _ = file.flush();
    }
  }

  pub fn save_refresh_token(env_file: &str, refresh_token: &str) {
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .append(false)
      .write(true)
      .open(env_file)
    {
      let data = format!("RefreshToken={}", refresh_token);

      if let Err(e) = file.write_all(data.as_bytes()) {
        panic!("{}", e);
      }
      let _ = file.flush();
    }
  }
}
