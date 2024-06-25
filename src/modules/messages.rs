use crate::{EventSubError, Reward};

#[derive(Debug, PartialEq)]
pub struct MessageData {
  pub user_id: String,
  pub message_id: String,
  pub username: String,
  pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum MessageType {
  AdBreakNotification(u32),
  ChatMessage(MessageData),
  CustomRedeem((String, String, Reward)),
  RawResponse(String),
  CustomSubscriptionResponse(String),
  SubscribeError(EventSubError),
  Error(EventSubError),
  Close,
}
