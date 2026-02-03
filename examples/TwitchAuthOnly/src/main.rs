use twitcheventsub_structs::prelude::Subscription;
use twitcheventsub_tokens::*;

fn main() {
  let mut token_builder = TokenHandler::builder().add_subscriptions(Subscription::recommended());
  match token_builder.build() {
    Ok(mut token) => {
      println!("Successfully got valid user token");
      // You can let it handle saving of the tokens
      token.save();

      // You have a few twitch api endpoints ready to be used.
      // If you want an expanded api callset, you can use
      // twitcheventsub-api crate
      let mut broadcaster_id = String::new();
      if let Ok(users) = token.get_users(vec![] as Vec<String>, vec![] as Vec<String>) {
        broadcaster_id = users.data[0].clone().id;
      }

      // You can run a simple command like this
      let mut result = token.send_chat_message(&broadcaster_id, "Test message from test example");

      if result.is_err() {
        // When you run any commands attached to token or
        // via Twitcheventsub-api, itll return a Result<_, TwitchApiError>
        // If you want it to handle regening user_tokens when they expire
        // Just pass the result into the regen_tokens_on_fail function
        // This will regen the tokens and reattempt the api call
        result = token.regen_tokens_on_fail(result);
      }

      if let Ok(_returned_data) = result {
        // If this api call returned any data
        // this is where youll get it
        // But since we did the send_chat_message endpoint
        // it returns nothing, so this is an empty string
      }

      // token.user_token is public if you want to manually use it.
    }
    Err(e) => match e {
      TokenBuilderError::ClientIdNotSet => {
        panic!("Client id not set withint the .env file. CLIENT_ID=INSERT_ID")
      }
      TokenBuilderError::ClientSecretNotSet => {
        panic!("Client secret not set within the .env file. CLIENT_SECRET=INSERT_SECRET")
      }
      TokenBuilderError::RedirectUrlIncorrect => {
        panic!(
          "Incorrect redirect url set. REDIRECT_URL=http://localhost:3000\nPlease match your url exactly to the one set in your twitch dev console."
        )
      }
      TokenBuilderError::EnvDoesntExist => {
        panic!("env file doesnt exist. Default: PROJECT_FOLDER/.secrets.env")
      }
      TokenBuilderError::InvalidClientIdOrSecret => panic!("Invalid id or secret"),
      TokenBuilderError::ManuallyInputAuthorisationCode => {
        println!("something needs to happen here")
      }
      TokenBuilderError::TwitchApiError(twitch_api_error) => {
        panic!("A network error occured: {:?}", twitch_api_error)
      }
      TokenBuilderError::InvalidUserToken => panic!("Invalid User token"),
    },
  }
}
