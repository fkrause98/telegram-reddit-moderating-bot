use teloxide::{
    dispatching::DispatcherBuilder,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
mod reddit;
type WebHandler = Endpoint<'static, DependencyMap, String>;
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    // let handler = dptree::entry().branch(Update::filter_message().endpoint(moderation_handler));
    // Dispatcher::builder(bot, handler)
    //     .enable_ctrlc_handler()
    //     .build()
    //     .dispatch()
    //     .await;
    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Print info about you")]
    Me,
    #[command(description = "Get the modqueue")]
    ModQueue,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Me => {
            let mut client = reddit::Client::new();
            if let Ok(about_me) = &client.reddit_request("/api/v1/me").await {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Fetched succesfully! Your user is {}",
                        about_me.get("name").unwrap()
                    ),
                )
                .await?
            } else {
                bot.send_message(msg.chat.id, "Error accessing token".to_string())
                    .await?
            }
        }
        Command::ModQueue => {
            let mut client = reddit::Client::new();
            if let Ok(Some(response)) = &client.get_modqueue(1).await {
                let mod_options = ["Approve", "Remove"]
                    .map(|string| InlineKeyboardButton::callback(string, string));
                bot.send_message(msg.chat.id, response[0].link_url.clone())
                    .reply_markup(InlineKeyboardMarkup::new([mod_options]))
                    .await?
            } else {
                bot.send_message(msg.chat.id, "Error accessing token".to_string())
                    .await?
            }
        }
    };

    Ok(())
}
