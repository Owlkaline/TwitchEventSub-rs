use log::LevelFilter;
//use simple_logging;

use crate::{modules::twitch_http::TwitchHttpRequest, Subscription};

pub const LOG_FILE: &str = "twitch_events.log";
pub const LOG_FILE_BUILDER: &str = "twitch_event_builder.log";

//pub fn log_info() {
//  let _ = simple_logging::log_to_file(LOG_FILE, LevelFilter::Info);
//}
//
//pub fn log_builder() {
//  let _ = simple_logging::log_to_file(LOG_FILE_BUILDER, LevelFilter::Info);
//}

#[derive(Debug, PartialEq)]
pub enum EventSubError {
  TokenMissingScope,
  TokenMissingSubscription(Subscription),
  TokenMissingUnimplementedSubscription(String),
  NoSubscriptionsRequested,
  AuthorisationError(String),
  WebsocketCreationFailed,
  MessageTooLong,
  UnhandledError(String),
  NoAccessTokenProvided,
  WriteError(String),
  // status 401 = invalid access token
  InvalidAccessToken(String),
  InvalidOauthToken(String),
  CurlFailed(curl::Error),
  ParseError(String),
  TokenRequiresRefreshing(TwitchHttpRequest),
  MaximumWebsocketTransmissionsExceeded(String),
}

#[derive(Debug)]
pub enum TwitchKeysError {
  ClientIdNotFound,
  ClientSecretNotFound,
}
