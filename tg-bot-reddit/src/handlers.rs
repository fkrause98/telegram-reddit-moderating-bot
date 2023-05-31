use crate::reddit::Client;
use anyhow::Result;
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
            log::info!("Handling moderation command");
            let mut client = Client::new();
            let modqueue = &client.get_modqueue(1).await?;
            match modqueue {
                Some(posts) => {
                    let post = &posts[0];
                    let mod_options = ["Approve", "Remove"].map(|string| {
                        let callback_data =
                            format!("{}+{}", string.to_lowercase(), &(post.post_id));
                        InlineKeyboardButton::callback(string, callback_data)
                    });
                    // TODO:
                    // Properly send messages, because it does not work
                    // properly with videos.
                    bot.send_message(
                        msg.chat.id,
                        format!("https://www.reddit.com{}", post.link_permalink),
                    )
                    .reply_markup(InlineKeyboardMarkup::new([mod_options]))
                    .await?;
                }
                None => {
                    bot.send_message(msg.chat.id, "No posts left to moderate".to_string())
                        .await?;
                }
            }
        }
        Err(_) => {
            log::info!("Command not found, ignoring");
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
    match (q.data, q.message) {
        (Some(callback_text), Some(Message { chat, .. })) => {
            log::info!("Handling moderation command...:");
            // TODO:
            // Properly send callback data, or store it
            // persistently inside a struct/disk.
            let (mod_option, post_id) = callback_text.split_once('+').unwrap();
            // TODO:
            // It's definitely not ok (not terrible though) to create a
            // client on each command.
            let client = Client::new();
            client
                .moderate_post(post_id, mod_option.try_into()?)
                .await?;
            // Tell telegram we've seen the query, and remove
            // the clock emoji.
            bot.answer_callback_query(q.id).await?;
            log::info!("Handled succesfully!");
            bot.send_message(chat.id, format!("Post {mod_option}d successfuly"))
                .await?;
            Ok(())
        }
        _ => Ok(()),
    }
}
