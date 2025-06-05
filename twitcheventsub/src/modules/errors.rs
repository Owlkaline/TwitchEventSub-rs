use twitcheventsub_api::TwitchApiError;

pub const LOG_FILE: &str = "twitch_events.log";

#[derive(Debug, PartialEq)]
pub enum EventSubError {
  WebSocketFailed(String),
  InvalidBroadcaster,
  WebsocketRestartFailed(String),
  TokenMissingUnimplementedSubscription(String),
  NoSubscriptionsRequested,
  WebsocketCreationFailed,
  MessageTooLong,
  UnhandledError(String),
  WriteError(String),
  CurlFailed(curl::Error),
  HttpFailed(String),
  ParseError(String),
  MaximumWebsocketTransmissionsExceeded(String),
  TwitchApiError(TwitchApiError),
}
