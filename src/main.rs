mod handler;

use crate::handler::Handler;
use dotenv::dotenv;
use serenity::all::{ActivityData, Client, GatewayIntents};
use std::env::var;

#[tokio::main]
async fn main() {
	dotenv().unwrap();

	let token = var("DISCORD_TOKEN").unwrap();
	let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(token, intents)
		.activity(ActivityData::custom("dirty clanker"))
		.event_handler(Handler)
		.await
		.unwrap();

	client.start().await.unwrap();
}
