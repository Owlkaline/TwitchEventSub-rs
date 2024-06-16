use crate::{EventSubError, Validation};
use curl::easy::{Easy, List};

#[derive(PartialEq)]
pub enum RequestType {
  Post(String),
}

pub enum AuthType {
  Bearer,
  OAuth,
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
    }
  }
}

pub struct TwitchHttp {
  url: String,
  header: Vec<String>,
  request_type: Option<RequestType>,
}

impl TwitchHttp {
  pub fn new<S: Into<String>>(url: S) -> TwitchHttp {
    TwitchHttp {
      url: url.into(),
      header: Vec::new(),
      request_type: None,
    }
  }

  #[must_use]
  pub fn full_auth<S: Into<String>, T: Into<String>>(
    self,
    access_token: S,
    client_id: T,
  ) -> TwitchHttp {
    self
      .header_authorisation(access_token, AuthType::Bearer)
      .header_client_id(client_id)
  }

  #[must_use]
  pub fn header_authorisation<S: Into<String>>(
    mut self,
    oauth: S,
    auth_type: AuthType,
  ) -> TwitchHttp {
    self.header.push(format!(
      "Authorization: {} {}",
      auth_type.to_string(),
      oauth.into()
    ));
    self
  }

  #[must_use]
  pub fn header_client_id<S: Into<String>>(mut self, client_id: S) -> TwitchHttp {
    self.header.push(format!("Client-Id: {}", client_id.into()));
    self
  }

  #[must_use]
  pub fn json_content(mut self) -> TwitchHttp {
    self.header.push(format!("Content-Type: application/json"));

    self
  }

  #[must_use]
  pub fn url_encoded_content(mut self) -> TwitchHttp {
    self
      .header
      .push(format!("Content-Type: application/x-www-form-urlencoded"));
    self
  }

  #[must_use]
  pub fn is_post<S: Into<String>>(mut self, data: S) -> TwitchHttp {
    self.request_type = Some(RequestType::Post(data.into()));
    self
  }

  pub fn run(&self) -> Result<String, EventSubError> {
    let mut data = Vec::new();

    let mut handle = Easy::new();
    {
      handle.url(&self.url).unwrap();
      if let Some(request) = &self.request_type {
        request.apply(&mut handle);
      }

      let mut headers = List::new();
      for header in &self.header {
        headers.append(header).unwrap();
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
        if let Ok(error) = serde_json::from_str::<Validation>(&e.to_string()) {
          if error.is_error() {
            return Err(EventSubError::InvalidOauthToken(error.error_msg()));
          }
        }
        return Err(EventSubError::CurlFailed(e));
      }
    }

    Ok(String::from_utf8_lossy(&data).to_string())
  }
}
