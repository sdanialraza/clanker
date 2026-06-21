use std::env::var;
use std::fs::read_to_string;
use std::process::Command;
use std::str::from_utf8;

use anyhow::{Result, ensure};
use dotenvy::dotenv;
use gemini_rust::{Gemini, Message};
use serenity::all::{ActivityData, Client, GatewayIntents};
use serenity::prelude::TypeMapKey;
use tokio::main;

mod handler;

use crate::handler::Handler;

struct State;

struct BotState {
	gemini_client: Gemini,

	system_prompt: String,
}

impl TypeMapKey for State {
	type Value = BotState;
}

struct History;

impl TypeMapKey for History {
	type Value = Vec<Message>;
}

#[main]

async fn main() -> Result<()> {
	let _ = dotenv();

	let token = var("DISCORD_TOKEN")?;
	let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

	let api_key = var("GEMINI_API_KEY")?;
	let gemini_client = Gemini::with_model(api_key, "models/gemini-3.1-flash-lite".to_string())?;

	let git_hash = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output()?;
	let git_url = Command::new("git").args(["remote", "get-url", "origin"]).output()?;

	ensure!(
		git_hash.status.success(),
		"Git error: {}",
		from_utf8(&git_hash.stderr)?.trim()
	);
	ensure!(
		git_url.status.success(),
		"Git error: {}",
		from_utf8(&git_url.stderr)?.trim()
	);

	let system_prompt = read_to_string("assets/prompt.txt")?
		.replace("$hash", from_utf8(&git_hash.clone().stdout)?.trim())
		.replace("$url", from_utf8(&git_url.clone().stdout)?.trim());

	let bot_state = BotState {
		gemini_client,
		system_prompt,
	};

	let mut client = Client::builder(token, intents)
		.activity(ActivityData::custom("Awaiting your prompts"))
		.event_handler(Handler)
		.type_map_insert::<State>(bot_state)
		.type_map_insert::<History>(Vec::new())
		.await?;

	client.start().await?;

	Ok(())
}
