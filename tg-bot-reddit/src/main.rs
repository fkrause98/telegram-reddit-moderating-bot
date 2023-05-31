use pretty_trace::*;
use teloxide::prelude::*;
mod handlers;
mod reddit;
mod reddit_schemas;
#[tokio::main]
async fn main() {
    PrettyTrace::new().on();
    pretty_env_logger::init();
    log::info!("Starting command bot");
    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handlers::message_handler))
        .branch(Update::filter_callback_query().endpoint(handlers::callback_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
