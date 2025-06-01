use std::{
  io::{stdin, BufRead, Read},
  net::TcpListener,
};

use curl::Error;
use request::*;
use twitcheventsub_structs::{NewAccessTokenResponse, Subscription};

pub const GET_USERS_URL: &str = "https://api.twitch.tv/helix/users";
pub const TWITCH_AUTHORISE_URL: &str = "https://id.twitch.tv/oauth2/";
pub const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";

mod request;

#[derive(Debug)]
pub enum TwitchApiError {
  CurlFailed(Error),
  MaximumWebsocketTransmissionsExceeded(String),
  TokenMissingSubscription(Box<Subscription>),
  TokenMissingUnimplementedSubscription(String),
  TokenRequiresRefreshing(Box<TwitchHttpRequest>),
  InvalidOauthToken(String),
  InvalidAuthorisationCode(String),
  BrowserError(String),
  HttpError(String),
  InputError(String),
}

pub fn get_users<T: Into<String>, I: Into<String>, S: Into<String>, V: Into<String>>(
  access_token: T,
  id: Vec<I>,
  login: Vec<S>,
  client_id: V,
) -> Result<String, TwitchApiError> {
  let mut url = RequestBuilder::new();
  if !id.is_empty() {
    url = url.add_key_value(
      "id",
      id.into_iter()
        .map(|id| id.into())
        .collect::<Vec<String>>()
        .join("&"),
    );
  }
  if !login.is_empty() {
    url = url.add_key_value(
      "login",
      login
        .into_iter()
        .map(|login| login.into())
        .collect::<Vec<String>>()
        .join("&"),
    );
  }
  let url = url.build(GET_USERS_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(access_token.into(), AuthType::Bearer)
    .header_client_id(client_id.into())
    .run()
}

pub fn get_implicit_grant_flow_user_token<S: Into<String>, T: Into<String>>(
  client_id: S,
  redirect_url: T,
  scopes: &Vec<Subscription>,
  auto_open_browser: bool,
  manual_code_input: bool,
) -> Result<String, TwitchApiError> {
  let redirect_url = redirect_url.into();

  let scope = &scopes
    .iter()
    .map(|s| s.required_scope())
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>()
    .join("+");

  let get_authorisation_code_request = format!(
    "{}authorize?response_type=token&client_id={}&redirect_uri={}&scope={}&force_verify=true",
    TWITCH_AUTHORISE_URL,
    client_id.into(),
    redirect_url.to_owned(),
    scope
  );

  match open_browser(
    get_authorisation_code_request,
    redirect_url,
    auto_open_browser,
    manual_code_input,
  ) {
    Ok(http_response) => {
      if http_response.contains("error") {
        Err(TwitchApiError::HttpError(format!("{}", http_response)))
      } else {
        let auth_code = if manual_code_input {
          http_response.trim()
        } else {
          http_response.split('&').collect::<Vec<_>>()[0]
            .split('=')
            .collect::<Vec<_>>()[1]
        };

        Ok(auth_code.to_owned())
      }
    }
    e => e,
  }
}

pub fn get_authorisation_code_grant_flow_user_token<S: Into<String>, T: Into<String>>(
  client_id: S,
  redirect_url: T,
  scopes: &Vec<Subscription>,
  auto_open_browser: bool,
  manual_code_input: bool,
) -> Result<String, TwitchApiError> {
  let redirect_url = redirect_url.into();

  let scope = &scopes
    .iter()
    .map(|s| s.required_scope())
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>()
    .join("+");

  let get_authorisation_code_request = format!(
    "{}authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&force_verify=true",
    TWITCH_AUTHORISE_URL,
    client_id.into(),
    redirect_url.to_owned(),
    scope
  );

  match open_browser(
    get_authorisation_code_request,
    redirect_url,
    auto_open_browser,
    manual_code_input,
  ) {
    Ok(http_response) => {
      if http_response.contains("error") {
        Err(TwitchApiError::HttpError(http_response))
      } else {
        if manual_code_input {
          Ok(http_response)
        } else {
          let auth_code = http_response.split('&').collect::<Vec<_>>()[0]
            .split('=')
            .collect::<Vec<_>>()[1];
          Ok(auth_code.to_string())
        }
      }
    }
    e => e,
  }
}

pub fn get_user_and_refresh_token_from_authorisation_code<
  S: Into<String>,
  T: Into<String>,
  V: Into<String>,
  W: Into<String>,
>(
  client_id: S,
  client_secret: T,
  authorisation_code: V,
  redirect_url: W,
) -> Result<(String, String), TwitchApiError> {
  let post_data = format!(
    "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
    client_id.into(),
    client_secret.into(),
    authorisation_code.into(),
    redirect_url.into()
  );

  process_token_query(post_data)
}

pub fn open_browser<S: Into<String>, T: Into<String>>(
  browser_url: S,
  redirect_url: T,
  auto_open_browser: bool,
  manual_code_input: bool,
) -> Result<String, TwitchApiError> {
  let browser_url = browser_url.into();

  if let Err(response) = TwitchHttpRequest::new(&browser_url).run() {
    return Err(response);
  }

  if auto_open_browser {
    if let Err(e) = open::that_detached(browser_url) {
      #[cfg(feature = "logging")]
      error!("Failed to open browser: {}", e);
      return Err(TwitchApiError::BrowserError(e.to_string()));
    }
  } else {
    println!(
      "Please visit the following link to have your token be authorised and generated:\n{}",
      browser_url
    );
  }

  let mut redirect_url = redirect_url.into().to_ascii_lowercase();

  if redirect_url.contains("http") && redirect_url.contains("/") {
    redirect_url = redirect_url
      .split('/')
      .collect::<Vec<_>>()
      .last()
      .unwrap()
      .to_string();
  }

  if manual_code_input {
    println!("Please input your access token:");
    let stdin = stdin();
    for line in stdin.lock().lines() {
      let code = line.unwrap();
      return Ok(code);
    }

    Err(TwitchApiError::InputError(
      "Failed to get user input.".to_string(),
    ))
  } else {
    println!("Starting local tcp listener for token generation");
    println!("Please click the blue redirect link if browser doesn't redirect.");
    let listener = TcpListener::bind(&redirect_url).expect("Failed to create tcp listener.");

    // accept connections and process them serially
    match listener.accept() {
      Ok((mut stream, _b)) => {
        let mut http_output = String::new();
        stream
          .read_to_string(&mut http_output)
          .expect("Failed to read tcp stream.");
        Ok(http_output)
      }
      Err(e) => Err(TwitchApiError::HttpError(e.to_string())),
    }
  }
}

fn process_token_query<S: Into<String>>(post_data: S) -> Result<(String, String), TwitchApiError> {
  TwitchHttpRequest::new(TWITCH_TOKEN_URL)
    .url_encoded_content()
    .is_post(post_data)
    .run()
    .and_then(|twitch_response| {
      serde_json::from_str::<NewAccessTokenResponse>(&twitch_response)
        .map_err(|_| TwitchApiError::InvalidAuthorisationCode(twitch_response))
        .map(|new_token_data| {
          (
            new_token_data.access_token,
            new_token_data.refresh_token.unwrap(),
          )
        })
    })
}
