use std::{
  io::ErrorKind,
  net::TcpStream,
  sync::mpsc::{Receiver as SyncReceiver, Sender as SyncSender},
  thread,
  time::{Duration, Instant},
};

#[cfg(feature = "logging")]
use log::warn;
#[cfg(feature = "logging")]
use log::{error, info};
use tungstenite::{connect, stream::MaybeTlsStream, Error, Message as NetworkMessage, WebSocket};
use twitcheventsub_api::TwitchHttpRequest;
use twitcheventsub_structs::prelude::{
  EventMessageType, GenericMessage, Subscription, TwitchEvent,
};
use twitcheventsub_tokens::TokenHandler;

use super::irc_bot::IRCChat;
use super::{bttv::BTTV, irc_bot};
use crate::{EventSubError, ResponseType, CONNECTION_EVENTS, SUBSCRIBE_URL};

#[allow(clippy::too_many_arguments)]
pub fn events(
  mut twitch_receiver: WebSocket<MaybeTlsStream<TcpStream>>, //Client<TlsStream<TcpStream>>>>,
  message_sender: SyncSender<ResponseType>,
  should_quit_receiver: SyncReceiver<bool>,
  subscriptions: Vec<Subscription>,
  mut custom_subscriptions: Vec<String>,
  mut tokens: TokenHandler,
  irc: Option<IRCChat>,
  bttv: BTTV,
  broadcasters_users_id: &str,
) {
  if subscriptions.iter().all(|s| s.is_permission_subscription()) {
    // Don't attempt eventsub things if no event sub events are being subscribed to
    #[cfg(feature = "logging")]
    error!("EventSub: no eventsub subscriptions chosen, exiting eventsub thread.");
    return;
  }

  use std::sync::mpsc::channel;

  use twitcheventsub_structs::prelude::{FragmentType, Fragments};

  use crate::modules::irc_bot::{IRCMessage, IRCResponse};

  let mut last_message = Instant::now();

  let mut is_reconnecting = false;

  let mut messages_from_irc = None;
  if let Some(irc) = irc {
    let (transmit_messages, receive_message) = channel();

    let _ = thread::spawn(move || {
      irc_bot::irc_thread(irc, transmit_messages);
    });

    messages_from_irc = Some(receive_message);
  }

  let mut irc_messages: Vec<(Instant, IRCMessage)> = Vec::new();

  loop {
    
    if let Ok(true) = should_quit_receiver.try_recv() {
      return;
    }

    let message = match twitch_receiver.read() {
      Ok(m) => m,
      Err(Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => {
        // shouldn't happen
        #[cfg(feature = "logging")]
        error!("EventSub: Would block");
        continue;
      }
      Err(Error::ConnectionClosed) | Err(Error::AlreadyClosed) => {
        #[cfg(feature = "logging")]
        error!("EventSub: Connected closed or already closed");
        let _ = twitch_receiver.send(NetworkMessage::Close(None));
        let _ = message_sender.send(ResponseType::Close);
        thread::sleep(Duration::from_secs(30));
        #[cfg(feature = "logging")]
        warn!("EventSub: Attempting reconnect.");
        let (new_client, _) = connect(CONNECTION_EVENTS)
          .expect("Failed to reconnect to new url after receiving reconnect message from twitch");
        twitch_receiver = new_client;
        last_message = Instant::now();
        is_reconnecting = false;
        continue;
      }
      Err(e) => {
        #[cfg(feature = "logging")]
        error!("EventSub: Read error: {}", e);
        thread::sleep(Duration::from_secs(5));
        continue;
      }
    };

    while let Some(irc_reciever) = &messages_from_irc {
      match irc_reciever.recv_timeout(Duration::ZERO) {
        Ok(IRCResponse::IRCMessage(msg)) => irc_messages.push((Instant::now(), msg)),
        _ => break,
      }
    }

    if last_message.elapsed().as_secs() > 60 {
      let _ = twitch_receiver.send(NetworkMessage::Close(None));
      thread::sleep(Duration::from_secs(5));
      #[cfg(feature = "logging")]
      error!("Messages not sent within the keep alive timeout restarting websocket");
      let (new_client, _) = connect(CONNECTION_EVENTS)
        .expect("Failed to reconnect to new url after receiving reconnect message from twitch");
      twitch_receiver = new_client;
      last_message = Instant::now();
      is_reconnecting = false;
      continue;
    }

    for i in (0..irc_messages.len()).rev() {
      if irc_messages[i].0.elapsed().as_secs() > 30 {
        irc_messages.remove(i);
      }
    }

    match message {
      NetworkMessage::Text(msg) => {
        #[cfg(feature = "only_raw_responses")]
        {
          let _ = message_sender.send(ResponseType::RawResponse(msg.to_string()));
        }

        let message = serde_json::from_str(msg.as_str());

        if let Err(e) = message {
          #[cfg(feature = "logging")]
          error!("EventSub: Unimplemented twitch response: {}\n{}", msg, e);

          let _ = message_sender.send(ResponseType::RawResponse(msg.to_string()));
          continue;
        }

        let message: GenericMessage = message.unwrap();

        match message.event_type() {
          EventMessageType::Welcome => {
            #[cfg(feature = "logging")]
            error!("EventSub: Welcome message!");
            let session_id = message.clone().payload.unwrap().session.unwrap().id;

            if !is_reconnecting {
              let user_token = tokens.user_token.clone();
              let token_user_id = tokens
                .get_token_user_id()
                .expect("Failed to get tokens user id.");
              let client_id = tokens.client_id.clone();

              let mut sub_data = subscriptions
                .iter()
                .filter_map(|s| {
                  s.construct_data(&session_id, broadcasters_users_id, &token_user_id)
                })
                .filter_map(|s| serde_json::to_string(&s).ok())
                .collect::<Vec<_>>();
              sub_data.append(&mut custom_subscriptions);

              #[cfg(feature = "logging")]
              error!("EventSub: Subscribing to events!");
              let failed_to_communicate_with_main_thread = sub_data
                .iter()
                .map(|sub_data| {
                  TwitchHttpRequest::new(SUBSCRIBE_URL)
                    .full_auth(&user_token, &client_id)
                    .json_content()
                    .is_post(sub_data)
                    .run()
                })
                .map(|a| tokens.regen_tokens_on_fail(a))
                .filter_map(Result::err)
                .any(|error| {
                  #[cfg(feature = "logging")]
                  error!("EventSub: {:?}", error);
                  let sent_msg = message_sender.send(ResponseType::Error(Box::new(
                    EventSubError::TwitchApiError(error),
                  )));
                  sent_msg.is_err()
                });

              if failed_to_communicate_with_main_thread {
                #[cfg(feature = "logging")]
                error!("EventSub: Failed to communicate with main thread, exiting!");
                return;
              }

              //twitch_keys = clone_twitch_keys;
              #[cfg(feature = "logging")]
              error!("Twitch Event loop is ready!");
              message_sender
                .send(ResponseType::Ready)
                .expect("Failed to send ready back to main thread.");
            }
            is_reconnecting = false;
            last_message = Instant::now();
          }
          EventMessageType::KeepAlive => {
            #[cfg(feature = "logging")]
            error!("EventSub: Keep alive: {}", last_message.elapsed().as_secs());
            last_message = Instant::now();
          }
          EventMessageType::Reconnect => {
            #[cfg(feature = "logging")]
            error!("EventSub: Twitch requested reconnection");
            let url = message
              .clone()
              .payload
              .unwrap()
              .session
              .unwrap()
              .reconnect_url
              .unwrap();

            is_reconnecting = true;
            let _ = twitch_receiver.send(NetworkMessage::Close(None));
            let (new_client, _) = connect(&url).expect(
              "Failed to reconnect to new url after recieving reconnect message from twitch.",
            );
            twitch_receiver = new_client;
          }
          EventMessageType::Notification => {
            #[cfg(feature = "only_raw_responses")]
            continue;

            last_message = Instant::now();
            let mut message = message.payload.unwrap().event.unwrap();

            if let TwitchEvent::ChatMessage(ref mut msg) = &mut message {
              for (_, irc_message) in irc_messages.iter() {
                if irc_message.display_name == msg.chatter.name &&
                  irc_message.message.contains(&msg.message.text)
                {
                  msg.returning_chatter = irc_message.returning_chatter;
                  msg.first_time_chatter = irc_message.first_time_chatter;
                  msg.moderator = msg
                    .badges
                    .iter()
                    .any(|badge| badge.set_id.contains("moderator"));
                  break;
                }
              }

              let mut fragments: Vec<Fragments> = Vec::new();
              for fragment in &mut msg.message.fragments {
                // Only check plain text for bttv emotes
                if fragment.kind == FragmentType::Text {
                  let mut new_fragment: Fragments = fragment.clone();
                  new_fragment.text = String::new();
                  let text_particles = fragment.text.split(' ').collect::<Vec<_>>();

                  for test_text in text_particles {
                    if bttv.emote_names.contains(&test_text.to_lowercase()) {
                      if !new_fragment.text.is_empty() {
                        fragments.push(new_fragment);
                      }

                      new_fragment = fragment.clone();
                      // is BTTV emote
                      new_fragment.kind = FragmentType::BttvEmote;
                      new_fragment.text = test_text.to_lowercase().to_string();

                      fragments.push(new_fragment);

                      new_fragment = fragment.clone();
                      new_fragment.text = String::new();
                    } else {
                      new_fragment.text = format!("{}{} ", new_fragment.text, test_text);
                    }
                  }

                  if !new_fragment.text.is_empty() {
                    fragments.push(new_fragment);
                  }
                } else {
                  fragments.push(fragment.clone());
                }
              }

              msg.message.fragments = fragments;
            }

            let _ = message_sender.send(ResponseType::Event(Box::new(message)));
          }
          EventMessageType::Unknown => {
            #[cfg(feature = "logging")]
            error!("EventSub: Unknown message type: {}", msg);
            last_message = Instant::now();
            //if !custom_subscriptions.is_empty() {
            let _ = message_sender.send(ResponseType::RawResponse(msg.to_string()));
            //}
          }
        }
      }
      NetworkMessage::Close(a) => {
        #[cfg(feature = "logging")]
        warn!("EventSub: Close message received: {:?}", a);
        // Got a close message, so send a close message and return
        let _ = twitch_receiver.send(NetworkMessage::Close(None));
        let _ = message_sender.send(ResponseType::Close);

        // This will trigger attempts to reconnect or resubscribe afterwards
        continue;
      }
      NetworkMessage::Ping(_) => {
        #[cfg(feature = "logging")]
        info!("EventSub: ping recieved");
        match twitch_receiver.send(NetworkMessage::Pong(Vec::new().into())) {
          // Send a pong in response
          Ok(()) => {}
          Err(e) => {
            #[cfg(feature = "logging")]
            error!(
              "EventSub: sending Pong Received an Error from Server: {:?}",
              e
            );
            continue;
          }
        }
      }
      nm => {
        #[cfg(feature = "logging")]
        info!("EventSub: Other network message recieved: {}", nm);
      }
    }
  }
}
