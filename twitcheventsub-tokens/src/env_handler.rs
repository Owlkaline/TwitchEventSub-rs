use std::{fs::OpenOptions, io::Write};

use env_file_reader::read_file;

pub struct EnvHandler;

impl EnvHandler {
  pub fn load_env(env_file: &str) -> Option<(String, String, String, String)> {
    dbg!(env_file);
    match read_file(env_file) {
      Ok(vars) => {
        let client_id = vars
          .get("CLIENT_ID")
          .map(String::clone)
          .unwrap_or(String::new());
        let client_secret = vars
          .get("CLIENT_SECRET")
          .map(String::clone)
          .unwrap_or(String::new());
        let redirect_url = vars
          .get("REDIRECT_URL")
          .map(String::clone)
          .unwrap_or(String::new());
        let twitch_id = vars
          .get("CLIENT_TWITCH_ID")
          .map(String::clone)
          .unwrap_or(String::new());

        Some((client_id, client_secret, redirect_url, twitch_id))
      }
      Err(e) => {
        dbg!(e);
        println!("No env file called {:?}", env_file);
        None
      }
    }
  }

  pub fn load_user_token_env(env_file: &str) -> Option<String> {
    match read_file(env_file) {
      Ok(vars) => {
        let user_token = &vars["UserToken"];

        Some(user_token.to_owned())
      }
      Err(_) => {
        println!("No env file called {:?}", env_file);
        None
      }
    }
  }

  pub fn load_refresh_token_env(env_file: &str) -> Option<String> {
    match read_file(env_file) {
      Ok(vars) => {
        let refresh_token = &vars["RefreshToken"];

        Some(refresh_token.to_owned())
      }
      Err(_) => {
        println!("No env file called {:?}", env_file);
        None
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
