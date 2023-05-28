use crate::reddit::Client;
use std::error::Error;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Me},
    utils::command::BotCommands,
};
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Get the modqueue")]
    ModQueue,
}
/// Parse the incoming command.
pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let text = msg.text().ok_or("No text on this message")?;
    match BotCommands::parse(text, me.username()) {
        Ok(Command::ModQueue) => {
            let mut client = Client::new();
            if let Ok(Some(response)) = &client.get_modqueue(5).await {
                let mod_options = ["Approve", "Remove"]
                    .map(|string| InlineKeyboardButton::callback(string, string));
                bot.send_message(msg.chat.id, response.first().unwrap().link_url.clone())
                    .reply_markup(InlineKeyboardMarkup::new([mod_options]))
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Error accessing token".to_string())
                    .await?;
            }
        }
        Err(_) => {
            bot.send_message(msg.chat.id, "Command not found").await?;
        }
    }
    Ok(())
}
/// When it receives a callback from a button it edits the message with all
/// those buttons writing a text with the selected Debian version.
///
/// **IMPORTANT**: do not send privacy-sensitive data this way!!!
/// Anyone can read data stored in the callback button.
pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(version) = q.data {
        let text = format!("You chose: {version}");

        // Tell telegram that we've seen this query, to remove 🕑 icons from the
        // clients. You could also use `answer_callback_query`'s optional
        // parameters to tweak what happens on the client side.
        bot.answer_callback_query(q.id).await?;

        // Edit text of the message to which the buttons were attached
        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }

        log::info!("You chose: {}", version);
    }

    Ok(())
}
