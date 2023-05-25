use std::collections::HashMap;
use teloxide::{prelude::*, utils::command::BotCommands};
mod reddit;
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
    #[command(description = "Debug")]
    Debug(String),
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}."))
                .await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?
        }
        Command::Debug(_string) => {
            let mut client = reddit::Client::new();
            if let Ok(auth_token) = client.get_token().await {
                let about_me = &client.reddit_request("/api/v1/me").await.unwrap();
                println!(
                    "The about me json: {}",
                    serde_json::to_string_pretty(about_me).unwrap()
                );
                bot.send_message(msg.chat.id, "Fetched succesfully!")
                    .await?
            } else {
                bot.send_message(msg.chat.id, format!("Error accessing token"))
                    .await?
            }
        }
    };

    Ok(())
}
