use curl::easy::{Easy, List};
#[cfg(feature = "logging")]
use log::{error, info};
use twitcheventsub_structs::{Subscription, Validation};

use crate::TwitchApiError;

#[derive(PartialEq, Clone, Debug)]
pub enum RequestType {
  Post(String),
  Delete,
  Patch(String),
}

#[derive(Clone, PartialEq, Debug)]
pub enum AuthType {
  Bearer,
  OAuth,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Header {
  Auth((AuthType, String)),
  ClientId(String),
  ContentJson,
  ContentUrlEncoded,
}

impl Header {
  pub fn generate(&self) -> String {
    match self {
      Header::Auth((auth_type, token)) => {
        format!("Authorization: {} {}", auth_type.to_string(), token)
      }
      Header::ClientId(id) => {
        format!("Client-Id: {}", id)
      }
      Header::ContentJson => {
        format!("Content-Type: application/json")
      }
      Header::ContentUrlEncoded => {
        format!("Content-Type: application/x-www-form-urlencoded")
      }
    }
  }
}

impl AuthType {
  pub fn to_string(&self) -> String {
    match self {
      AuthType::Bearer => "Bearer",
      AuthType::OAuth => "OAuth",
    }
    .into()
  }
}

impl RequestType {
  pub fn apply(&self, handle: &mut Easy) {
    match self {
      RequestType::Post(data) => {
        handle.post(true).unwrap();
        handle.post_fields_copy(data.as_bytes()).unwrap();
      }
      RequestType::Delete => {
        let _ = handle.custom_request("DELETE");
      }
      RequestType::Patch(data) => {
        handle.put(true).unwrap();
        handle.post_fields_copy(data.as_bytes()).unwrap();
        let _ = handle.custom_request("PATCH");
      }
    }
  }
}

pub struct RequestBuilder {
  data: Vec<(String, String)>,
}

impl RequestBuilder {
  pub fn new() -> RequestBuilder {
    RequestBuilder { data: Vec::new() }
  }

  pub fn add_key_value<S: Into<String>, T: Into<String>>(
    mut self,
    key: S,
    value: T,
  ) -> RequestBuilder {
    self.data.push((key.into(), value.into()));
    self
  }

  pub fn build<S: Into<String>>(self, url: S) -> String {
    let mut request = url.into();

    if !self.data.is_empty() {
      request = format!("{}?", request);
    }

    for (key, value) in self.data {
      request = format!("{}&{}={}", request, key, value);
    }

    request
  }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TwitchHttpRequest {
  url: String,
  headers: Vec<Header>,
  request_type: Option<RequestType>,
}

impl TwitchHttpRequest {
  pub fn new<S: Into<String>>(url: S) -> TwitchHttpRequest {
    TwitchHttpRequest {
      url: url.into(),
      headers: Vec::new(),
      request_type: None,
    }
  }

  #[must_use]
  pub fn full_auth<S: Into<String>, T: Into<String>>(
    self,
    access_token: S,
    client_id: T,
  ) -> TwitchHttpRequest {
    self
      .header_authorisation(access_token, AuthType::Bearer)
      .header_client_id(client_id)
  }

  #[must_use]
  pub fn add_header(mut self, header: Header) -> TwitchHttpRequest {
    self.headers.push(header);
    self
  }

  #[must_use]
  pub fn header_authorisation<S: Into<String>>(
    mut self,
    token: S,
    auth_type: AuthType,
  ) -> TwitchHttpRequest {
    self.headers.push(Header::Auth((auth_type, token.into())));
    self
  }

  #[must_use]
  pub fn header_client_id<S: Into<String>>(mut self, client_id: S) -> TwitchHttpRequest {
    self.headers.push(Header::ClientId(client_id.into()));
    self
  }

  #[must_use]
  pub fn json_content(mut self) -> TwitchHttpRequest {
    self.headers.push(Header::ContentJson);
    self
  }

  #[must_use]
  pub fn url_encoded_content(mut self) -> TwitchHttpRequest {
    self.headers.push(Header::ContentUrlEncoded);
    self
  }

  #[must_use]
  pub fn is_delete(mut self) -> TwitchHttpRequest {
    self.request_type = Some(RequestType::Delete);
    self
  }

  #[must_use]
  pub fn is_post<S: Into<String>>(mut self, data: S) -> TwitchHttpRequest {
    self.request_type = Some(RequestType::Post(data.into()));
    self
  }

  #[must_use]
  pub fn is_patch<S: Into<String>>(mut self, data: S) -> TwitchHttpRequest {
    self.request_type = Some(RequestType::Patch(data.into()));
    self
  }

  pub fn update_token(&mut self, new_token: &str) {
    for header in &mut self.headers {
      if let Header::Auth((_, ref mut token)) = header {
        *token = new_token.into();
        break;
      }
    }
  }

  pub fn run(&self) -> Result<String, TwitchApiError> {
    let mut data = Vec::new();

    #[cfg(feature = "logging")]
    info!("Running curl command with:");
    #[cfg(feature = "logging")]
    info!("    url: {}", self.url);
    let mut handle = Easy::new();
    {
      handle.url(&self.url).unwrap();
      if let Some(request) = &self.request_type {
        request.apply(&mut handle);
      }

      let mut headers = List::new();
      for header in &self.headers {
        headers.append(&header.generate()).unwrap();
      }

      handle.http_headers(headers).unwrap();

      let mut handle = handle.transfer();
      // getting data back
      // idk why its called write function
      // that silly
      // we are reading whats coming back
      let _ = handle.write_function(|new_data| {
        data.extend_from_slice(new_data);
        Ok(new_data.len())
      });

      if let Err(e) = handle.perform() {
        #[cfg(feature = "logging")]
        error!("Curl error: {}", e);
        return Err(TwitchApiError::CurlFailed(e));
      }
    }

    let data = String::from_utf8_lossy(&data).to_string();
    if let Ok(error) = serde_json::from_str::<Validation>(&data) {
      if error.is_error() {
        if error.status.unwrap() == 429 {
          return Err(TwitchApiError::MaximumWebsocketTransmissionsExceeded(
            error.error_msg(),
          ));
        }
        if error.status.unwrap() == 401 {
          // Regen access token
          // Re run the query

          let error = error.message.unwrap();
          if error.contains("Missing scope") {
            let scope = error.split_whitespace().nth(2).unwrap();
            if let Some(missing_subscription) = Subscription::from_scope(scope) {
              #[cfg(feature = "logging")]
              error!(
                "Token missing subscription: Subscription::{:?}",
                missing_subscription
              );
              return Err(TwitchApiError::TokenMissingSubscription(Box::new(
                missing_subscription,
              )));
            } else {
              #[cfg(feature = "logging")]
              error!("Token missing unimplemented subscription: {}", scope);
              return Err(TwitchApiError::TokenMissingUnimplementedSubscription(
                scope.to_owned(),
              ));
            }
          } else {
            #[cfg(feature = "logging")]
            info!("Token requires refresing, debug: {:?}", error);
            return Err(TwitchApiError::TokenRequiresRefreshing(Box::new(
              self.to_owned(),
            )));
          }
        }
        #[cfg(feature = "logging")]
        error!("Unhandled error: {}, {}", self.url, error.error_msg());
        return Err(TwitchApiError::InvalidOauthToken(error.error_msg()));
      }
    }

    Ok(data)
  }
}
