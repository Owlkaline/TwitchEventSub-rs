use std::{io::stdin, process::exit};

use twitcheventsub_api::{
  get_user_and_refresh_token_from_authorisation_code, validate_token, TwitchApiError,
};
use twitcheventsub_structs::prelude::Subscription;

use crate::{env_handler::EnvHandler, TokenHandler};

#[derive(Debug)]
pub enum TokenBuilderError {
  ClientIdNotSet,
  ClientSecretNotSet,
  RedirectUrlIncorrect,
  EnvDoesntExist,
  InvalidClientIdOrSecret,
  ManuallyInputAuthorisationCode,
  TwitchApiError(TwitchApiError),
  InvalidUserToken,
}

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

pub fn get_input() -> String {
  let mut user_input = String::new();

  while user_input.is_empty() {
    let _ = stdin().read_line(&mut user_input);
    user_input = user_input.trim().to_lowercase();
  }

  user_input
}

#[derive(Debug)]
pub struct TokenHandlerBuilder {
  env_file: String,
  env_user_token_file: String,
  env_refresh_token_file: String,
  use_specific_account: Option<String>,
  is_bot: bool,
  override_redirect_url: Option<String>,
  subscriptions: Vec<Subscription>,
}

impl Default for TokenHandlerBuilder {
  fn default() -> Self {
    TokenHandlerBuilder {
      env_file: String::from(".secrets.env"),
      env_user_token_file: String::from(".user_token"),
      env_refresh_token_file: String::from(".refresh_token"),
      use_specific_account: None,
      is_bot: false,
      override_redirect_url: None,
      subscriptions: Vec::new(),
    }
  }
}

impl TokenHandlerBuilder {
  pub fn new() -> TokenHandlerBuilder {
    TokenHandlerBuilder::default()
  }

  pub fn is_bot(mut self) -> TokenHandlerBuilder {
    self.is_bot = true;
    self
  }

  pub fn env_file(mut self, file: &str) -> TokenHandlerBuilder {
    self.env_file = file.into();
    self
  }

  pub fn user_token_env_file(mut self, file: &str) -> TokenHandlerBuilder {
    self.env_user_token_file = file.into();
    self
  }

  pub fn refresh_token_env_file(mut self, file: &str) -> TokenHandlerBuilder {
    self.env_refresh_token_file = file.into();
    self
  }

  pub fn use_twitch_account(mut self, username: &str) -> TokenHandlerBuilder {
    self.use_specific_account = Some(String::from(username));
    self
  }

  pub fn override_redirect_url(mut self, redirect_url: &str) -> TokenHandlerBuilder {
    self.override_redirect_url = Some(redirect_url.to_owned());
    self
  }

  pub fn add_subscriptions<I: IntoIterator<Item = Subscription>>(
    mut self,
    subs: I,
  ) -> TokenHandlerBuilder {
    self.subscriptions.extend(subs);
    self
  }

  fn create_basic_token(&self) -> TokenHandler {
    TokenHandler {
      user_token: String::new(),
      refresh_token: String::new(),
      redirect_url: self.override_redirect_url.clone().unwrap_or_default(),
      client_id: String::new(),
      client_secret: String::new(),
      client_twitch_id: self.use_specific_account.clone().unwrap_or_default(),
      env: self.env_file.clone(),
      user_token_env: self.env_user_token_file.clone(),
      refresh_token_env: self.env_refresh_token_file.clone(),
      subscriptions: Vec::new(),
    }
  }

  //pub fn build_from_env_only(&mut self) -> Option<TokenHandler> {
  //  let env_file = self.env_file.clone();

  //  let mut partial_tokens = TokenHandler {
  //    user_token: String::new(),
  //    refresh_token: String::new(),
  //    redirect_url: self.override_redirect_url.clone().unwrap_or(String::new()),
  //    client_id: String::new(),
  //    client_secret: String::new(),
  //    client_twitch_id: self.use_specific_account.clone().unwrap_or(String::new()),
  //    user_token_env: self.env_user_token_file.clone(),
  //    refresh_token_env: self.env_refresh_token_file.clone(),
  //  };

  //  if let Some((temp_client_id, temp_client_secret, temp_redirect_url, temp_twitch_id)) =
  //    EnvHandler::load_env(&env_file)
  //  {
  //    if !temp_client_id.is_empty() {
  //      partial_tokens.client_id = temp_client_id.to_owned();
  //    }
  //    if !temp_client_secret.is_empty() {
  //      partial_tokens.client_secret = temp_client_secret.to_owned();
  //    }
  //    if !temp_redirect_url.is_empty() && self.override_redirect_url.is_none() {
  //      partial_tokens.redirect_url = temp_redirect_url.to_owned();
  //    }
  //    if !temp_twitch_id.is_empty() && partial_tokens.client_twitch_id.is_empty() {
  //      partial_tokens.client_twitch_id = temp_twitch_id.to_owned();
  //    }

  //    Some(partial_tokens)
  //  } else {
  //    None
  //  }
  //}

  //pub fn generate_user_tokens(
  //  &self,
  //  mut partial_tokens: TokenHandler,
  //) -> Result<TokenHandler, TwitchApiError> {
  //  if let Some(user_token) = EnvHandler::load_user_token_env(&partial_tokens.user_token_env) {
  //    partial_tokens.user_token = user_token;
  //    partial_tokens.refresh_token =
  //      EnvHandler::load_refresh_token_env(&partial_tokens.refresh_token_env)
  //        .unwrap_or(String::new());

  //    // Check if theres already user and refresh tokens
  //    if let Ok(user_id) = partial_tokens.get_token_user_id() {
  //      partial_tokens.client_twitch_id = user_id;
  //      return Ok(partial_tokens)
  //    }
  //  }

  //  let (user_token, refresh_token) = get_user_and_refresh_tokens(
  //    &partial_tokens.client_id,
  //    &partial_tokens.client_secret,
  //    &partial_tokens.redirect_url,
  //    &if self.is_bot {
  //      Subscription::get_subscriptions_for_bot()
  //    } else {
  //      Vec::with_capacity(0)
  //    },
  //    true,
  //    false,
  //  );

  //  partial_tokens.user_token = user_token;
  //  partial_tokens.refresh_token = refresh_token;
  //  if let Ok(user_id) = partial_tokens.get_token_user_id() {
  //    partial_tokens.client_twitch_id = user_id;
  //  }

  //  EnvHandler::save_user_token(&partial_tokens.user_token_env, &partial_tokens.user_token);
  //  if !partial_tokens.refresh_token.is_empty() {
  //    EnvHandler::save_refresh_token(
  //      &partial_tokens.refresh_token_env,
  //      &partial_tokens.refresh_token,
  //    );
  //  }

  //  return Ok(partial_tokens);
  //}

  pub fn build(&mut self) -> Result<TokenHandler, TokenBuilderError> {
    let mut token = self.create_basic_token();
    EnvHandler::load_env(&mut token)?;
    EnvHandler::load_user_token_env(&mut token);
    EnvHandler::load_refresh_token_env(&mut token);

    if !token.refresh_token.is_empty() {
      match validate_token(&token.user_token) {
        Ok(validation) => {
          if validation.status.is_none() {
            match token.get_users(Vec::new() as Vec<String>, Vec::new() as Vec<String>) {
              Ok(users) => token.client_twitch_id = users.data[0].id.clone(),
              Err(api_error) => return Err(TokenBuilderError::TwitchApiError(api_error)),
            }
            // is okay
            return Ok(token);
          } else {
            if token.generate_user_token_from_refresh_token().is_ok() {
              token.save();
              return Ok(token);
            }
          }
        }
        Err(error) => {
          println!("Error {:?}", error)
        }
      }
      // if let Ok(validation) = validate_token(&token.client_id) {
      //   println!("{:?}", validation);
      //   if validation.status.is_none() {
      //     // is okay
      //     return Ok(token);
      //   } else {
      // if token.generate_user_token_from_refresh_token().is_ok() {
      //   token.save();
      //   return Ok(token);
      // }
      // }
      // }
    }

    if self.is_bot {
      self
        .subscriptions
        .extend(Subscription::get_subscriptions_for_bot());
    }
    if let Some(err) = token.generate_user_and_refresh_tokens(&self.subscriptions) {
      return Err(err);
    }

    match token.get_users(Vec::new() as Vec<String>, Vec::new() as Vec<String>) {
      Ok(users) => token.client_twitch_id = users.data[0].id.clone(),
      Err(api_error) => return Err(TokenBuilderError::TwitchApiError(api_error)),
    }

    Ok(token)
  }

  //pub fn build(&mut self) -> TokenHandler {
  //  let env_file = self.env_file.clone();

  //  let mut partial_tokens = TokenHandler {
  //    user_token: String::new(),
  //    refresh_token: String::new(),
  //    redirect_url: String::new(),
  //    client_id: String::new(),
  //    client_secret: String::new(),
  //    client_twitch_id: self.use_specific_account.clone().unwrap_or(String::new()),
  //    user_token_env: self.env_user_token_file.clone(),
  //    refresh_token_env: self.env_refresh_token_file.clone(),
  //  };

  //  if let Some(token) = self.build_from_env_only() {
  //    partial_tokens = token;
  //  } else {
  //    println!("Would you like to automatically create env file? Y/N");

  //    if !create_yn_prompt() {
  //      exit(0);
  //    }
  //  }

  //  let mut updates = false;

  //  if partial_tokens.client_id.is_empty() {
  //    println!(
  //      "You can get your Client id and secret from https://dev.twitch.tv/console/apps/create"
  //    );
  //    println!("Input your Client Id:");
  //    partial_tokens.client_id = get_input();
  //    updates = true;
  //  }

  //  if partial_tokens.client_secret.is_empty() {
  //    println!("Input your Client Secret:");
  //    partial_tokens.client_secret = get_input();

  //    updates = true;
  //  }

  //  if updates {
  //    println!("Would you like to save these into the {} Y/N", env_file);
  //    let save_details = create_yn_prompt();

  //    if save_details {
  //      EnvHandler::save_env(
  //        &env_file,
  //        &partial_tokens.client_id,
  //        &partial_tokens.client_secret,
  //        &partial_tokens.client_twitch_id,
  //        &partial_tokens.redirect_url,
  //      );
  //    }
  //  }

  //  let mut has_redirect_url = true;

  //  if partial_tokens.redirect_url.is_empty() {
  //    println!("Please input redirect url:");
  //    partial_tokens.redirect_url = get_input();
  //    has_redirect_url = false;
  //  }

  //  // if no refresh token force recreation of token flows
  //  if has_redirect_url {
  //    if let Some(user_token) = EnvHandler::load_user_token_env(&partial_tokens.user_token_env) {
  //      partial_tokens.user_token = user_token;
  //      partial_tokens.refresh_token =
  //        EnvHandler::load_refresh_token_env(&partial_tokens.refresh_token_env)
  //          .unwrap_or(String::new());

  //      // Check if theres already user and refresh tokens
  //      if let Ok(user_id) = partial_tokens.get_token_user_id() {
  //        // This means it is a new client id/secrets so should redo flow
  //        if partial_tokens.client_twitch_id != user_id {
  //          println!("client twitch id and user id are different");
  //        } else {
  //          return partial_tokens;
  //        }
  //      }
  //    }
  //  }

  //  println!("Do you want to automatically open the link? Y/N");
  //  let open_browser = create_yn_prompt();

  //  let (user_token, refresh_token) = get_user_and_refresh_tokens(
  //    &partial_tokens.client_id,
  //    &partial_tokens.client_secret,
  //    &partial_tokens.redirect_url,
  //    &if self.is_bot {
  //      Subscription::get_subscriptions_for_bot()
  //    } else {
  //      Vec::with_capacity(0)
  //    },
  //    open_browser,
  //    true,
  //  );

  //  partial_tokens.user_token = user_token;
  //  partial_tokens.refresh_token = refresh_token;

  //  EnvHandler::save_user_token(&partial_tokens.user_token_env, &partial_tokens.user_token);
  //  if !partial_tokens.refresh_token.is_empty() {
  //    EnvHandler::save_refresh_token(
  //      &partial_tokens.refresh_token_env,
  //      &partial_tokens.refresh_token,
  //    );
  //  }

  //  partial_tokens.client_twitch_id = match partial_tokens.get_token_user_id() {
  //    Ok(id) => id,
  //    Err(e) => {
  //      panic!("{:?}", e);
  //    }
  //  };

  //  EnvHandler::save_env(
  //    &env_file,
  //    &partial_tokens.client_id,
  //    &partial_tokens.client_secret,
  //    &partial_tokens.client_twitch_id,
  //    &partial_tokens.redirect_url,
  //  );

  //  partial_tokens
  //}
}

pub fn generate_authorisation_code(
  client_id: &str,
  redirect_url: &str,
  scopes: &[Subscription],
  open_browser: bool,
) -> Result<String, TokenBuilderError> {
  match twitcheventsub_api::get_authorisation_code_grant_flow_user_token(
    client_id,
    redirect_url,
    scopes,
    open_browser,
    redirect_url.contains("localhost") ||
      redirect_url.contains("127.0.0.1") ||
      redirect_url.contains("0.0.0.0") ||
      redirect_url.contains("::1") ||
      redirect_url.contains("0:0:0:0:0:0:0:1"),
  ) {
    Ok(potential_code) => {
      if let Some(code) = potential_code {
        Ok(code)
      } else {
        Err(TokenBuilderError::ManuallyInputAuthorisationCode)
      }
    }
    Err(e) => Err(TokenBuilderError::TwitchApiError(e)),
  }

  //  match twitcheventsub_api::get_user_and_refresh_token_from_authorisation_code(
  //    client_id,
  //    client_secret,
  //    code,
  //    redirect_url,
  //  ) {
  //    Ok(tokens) => tokens,
  //    Err(e) => {
  //      panic!("{:?}", e);
  //    }
  //  }
}
