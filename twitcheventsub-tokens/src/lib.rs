use std::{
  fs::{self, OpenOptions},
  io::{stdin, Write},
  process::exit,
};

use env_file_reader::read_file;
use env_handler::EnvHandler;
use log::{error, info, warn};
use twitcheventsub_api::{self, TwitchApiError};
use twitcheventsub_structs::{UserData, UserDataSet};

mod env_handler;

fn create_yn_prompt() -> bool {
  let mut user_input = String::new();
  loop {
    let _ = stdin().read_line(&mut user_input);
    user_input = user_input.to_lowercase();

    if user_input.contains("n") {
      return false;
    } else if user_input.contains("y") {
      return true;
    }
  }
}

fn get_input() -> String {
  let mut user_input = String::new();
  let _ = stdin().read_line(&mut user_input);
  user_input = user_input.trim().to_lowercase();

  return user_input
}

pub struct TokenHandlerBuilder {
  env_file: String,
  env_user_token_file: String,
  env_refresh_token_file: String,
  redirect_url: String,
  use_implicit_flow: bool,
}

impl Default for TokenHandlerBuilder {
  fn default() -> Self {
    TokenHandlerBuilder {
      env_file: String::from(".secrets.env"),
      env_user_token_file: String::from(".user_token"),
      env_refresh_token_file: String::from(".refresh_token"),
      redirect_url: String::from("http://localhost:3000"),
      use_implicit_flow: false,
    }
  }
}

impl TokenHandlerBuilder {
  pub fn new() -> TokenHandlerBuilder {
    TokenHandlerBuilder::default()
  }

  pub fn env_file(mut self, file: String) -> TokenHandlerBuilder {
    self.env_file = file;
    self
  }

  pub fn user_token_env_file(mut self, file: String) -> TokenHandlerBuilder {
    self.env_user_token_file = file;
    self
  }

  pub fn refresh_token_env_file(mut self, file: String) -> TokenHandlerBuilder {
    self.env_refresh_token_file = file;
    self
  }

  pub fn build(&mut self) -> TokenHandler {
    let env_file = self.env_file.clone();
    let user_env_file = self.env_user_token_file.clone();
    let refresh_env_file = self.env_refresh_token_file.clone();
    let mut client_id: String = String::new();
    let mut client_secret: String = String::new();
    let mut client_twitch_id: String = String::new();
    let mut redirect_url: String = String::new();

    if let Some((temp_client_id, temp_client_secret, temp_redirect_url, temp_twitch_id)) =
      EnvHandler::load_env(&env_file)
    {
      if !temp_client_id.is_empty() {
        client_id = temp_client_id.to_owned();
      }
      if !temp_client_secret.is_empty() {
        client_secret = temp_client_secret.to_owned();
      }
      if !temp_redirect_url.is_empty() {
        redirect_url = temp_redirect_url.to_owned();
      }
      if !temp_twitch_id.is_empty() {
        client_twitch_id = temp_twitch_id.to_owned();
      }
    } else {
      println!("Would you like to automatically create env file? Y/N");

      if !create_yn_prompt() {
        exit(0);
      }
    }

    let mut updates = false;

    if client_id.is_empty() {
      println!(
        "You can get your Client id and secret from https://dev.twitch.tv/console/apps/create"
      );
      while client_id.is_empty() {
        println!("Client id: ");
        client_id = get_input();
      }
      updates = true;
    }

    while client_secret.is_empty() {
      println!("Client secret: ");
      client_secret = get_input();

      updates = true;
    }

    if updates {
      println!("Would you like to save these into the default .secrets.env? Y/N");
      let save_details = create_yn_prompt();

      if save_details {
        EnvHandler::save_env(
          &env_file,
          &client_id,
          &client_secret,
          &client_twitch_id,
          &redirect_url,
        );
      }
    }

    let mut has_redirect_url = true;

    while redirect_url.is_empty() {
      println!("Redirect url: ");
      redirect_url = get_input();
      has_redirect_url = false;
    }

    // if no refresh token force recreation of token flows
    if has_redirect_url {
      if let Some(user_token) = EnvHandler::load_user_token_env(&user_env_file) {
        // Check if theres already user and refresh tokens
        if let Ok(client_id) = get_token_user_id(&client_id, &user_token) {
          // This means it is a new client id/secrets so should redo flow
          if client_twitch_id != client_id {
            // redo
          } else {
            if let Ok(client_id) = get_token_user_id(&client_id, &user_token) {
              let refresh_token =
                EnvHandler::load_refresh_token_env(&refresh_env_file).unwrap_or(String::new());

              return TokenHandler {
                user_token,
                refresh_token,

                client_id,
                client_secret,
                client_twitch_id,
              };
            }
          }
        }
      }
    }

    println!("Do you want to automatically open the link? Y/N");
    let open_browser = create_yn_prompt();

    let (user_token, refresh_token) = get_user_and_refresh_tokens(
      self.use_implicit_flow,
      &client_id,
      &client_secret,
      &redirect_url,
      open_browser,
    );

    EnvHandler::save_user_token(&user_env_file, &user_token);
    if !refresh_token.is_empty() {
      EnvHandler::save_refresh_token(&refresh_env_file, &refresh_token);
    }

    client_twitch_id = match get_token_user_id(&client_id, &user_token) {
      Ok(id) => id,
      Err(e) => {
        panic!("{:?}", e);
      }
    };

    EnvHandler::save_env(
      &env_file,
      &client_id,
      &client_secret,
      &client_twitch_id,
      &redirect_url,
    );

    TokenHandler {
      user_token,
      refresh_token,

      client_id,
      client_secret,
      client_twitch_id,
    }
  }
}

fn get_token_user_id(client_id: &str, user_token: &str) -> Result<String, TwitchApiError> {
  twitcheventsub_api::get_users(
    user_token,
    vec![] as Vec<String>,
    vec![] as Vec<String>,
    client_id.to_owned(),
  )
  .and_then(|user| {
    if let Ok(user) = serde_json::from_str::<UserDataSet>(&user) {
      Ok(user.data[0].id.to_owned())
    } else {
      Err(TwitchApiError::InputError(
        "Failed to Deserialise user information from get_user endpoint".to_owned(),
      ))
    }
  })

  //match twitcheventsub_api::get_users(
  //  user_token,
  //  vec![] as Vec<String>,
  //  vec![] as Vec<String>,
  //  client_id.to_owned(),
  //) {
  //  Ok(user) => {
  //    if let Ok(user) = serde_json::from_str::<UserDataSet>(&user) {
  //      user.data[0].id.to_owned()
  //    } else {
  //      panic!("Failed to Deserialise user information from get_user endpoint")
  //    }
  //  }
  //  Err(e) => {
  //    panic!("{:?}", e);
  //  }
  //}
}

fn get_user_and_refresh_tokens(
  use_implicit_flow: bool,
  client_id: &str,
  client_secret: &str,
  redirect_url: &str,
  open_browser: bool,
) -> (String, String) {
  // Doesnt need client secret
  if use_implicit_flow {
    let user_token = match twitcheventsub_api::get_implicit_grant_flow_user_token(
      client_id,
      redirect_url,
      &Vec::with_capacity(0),
      open_browser,
      true,
    ) {
      Ok(code) => code,
      Err(e) => {
        panic!("{:?}", e);
      }
    };

    (user_token, String::new())
  } else {
    // Does need client secret
    println!("Are you running this localling and your redirect url is localhost? Y/N");
    let manual_input = !create_yn_prompt();

    let code = match twitcheventsub_api::get_authorisation_code_grant_flow_user_token(
      client_id,
      redirect_url,
      &Vec::with_capacity(0),
      open_browser,
      manual_input,
    ) {
      Ok(code) => code,
      Err(e) => {
        panic!("{:?}", e);
      }
    };

    match twitcheventsub_api::get_user_and_refresh_token_from_authorisation_code(
      client_id,
      client_secret,
      code,
      redirect_url,
    ) {
      Ok(tokens) => tokens,
      Err(e) => {
        panic!("{:?}", e);
      }
    }
  }
}

#[derive(Default)]
pub struct TokenHandler {
  pub user_token: String,
  pub refresh_token: String,

  pub client_id: String,
  pub client_secret: String,
  pub client_twitch_id: String,
}

impl TokenHandler {
  pub fn builder() -> TokenHandlerBuilder {
    TokenHandlerBuilder::default()
  }

  pub fn new() -> TokenHandler {
    TokenHandler::default()
  }
}
