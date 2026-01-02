use std::{env, process::Command};

use anyhow::{Error, Result};
use reqwest::{Client, Url};
use serenity::all::{Context, GuildId, Message};
use tokio::fs;

use crate::model::{RequestBody, RequestContent, RequestImageUrl, RequestMessage, ResponseBody};

pub async fn body(ctx: &Context, guild: GuildId) -> Result<RequestBody> {
	let child_stdout = Command::new("git")
		.arg("rev-parse")
		.arg("HEAD")
		.spawn()
		.unwrap()
		.wait_with_output()
		.unwrap()
		.stdout;

	let commit_hash = String::from_utf8(child_stdout).unwrap();

	let mut messages = Vec::new();

	let content = "The following emojis are available to you. Try to avoid unicode emojis.";
	messages.push(RequestMessage::developer(content.into()));

	for emoji in ctx.http.get_emojis(guild).await? {
		let content = vec![
			RequestContent::text(emoji.to_string()),
			RequestContent::image_url(RequestImageUrl { url: emoji.url() }),
		];

		messages.push(RequestMessage::user(content, "emoji-list".into()));
	}

	let content = fs::read_to_string("assets/prompt.txt")
		.await?
		.replace("${user_id}", env::var("DISCORD_APPLICATION_ID").unwrap().as_str())
		.replace("${git_commit}", commit_hash.as_str());

	messages.push(RequestMessage::developer(content));

	let model = env::var("OPENAI_MODEL")?;
	Ok(RequestBody { messages, model })
}

pub fn parse(message: &Message) -> RequestMessage {
	let mut images = Vec::new();

	for attachment in &message.attachments {
		if attachment.dimensions().is_some() {
			images.push(attachment.url.clone());
		}
	}

	for embed in &message.embeds {
		if let Some(image) = &embed.image {
			images.push(image.url.clone());
		}

		if let Some(thumbnail) = &embed.thumbnail {
			images.push(thumbnail.url.clone());
		}
	}

	let mut content = Vec::new();

	if !message.content.is_empty() {
		content.push(RequestContent::text(message.content.clone()));
	}

	for url in images {
		content.push(RequestContent::image_url(RequestImageUrl { url }));
	}

	RequestMessage::user(content, message.author.name.clone())
}

pub async fn post(body: &mut RequestBody, message: &Message, reply: Option<&Message>) -> Result<String> {
	if let Some(message) = reply {
		let content = "Use the following message as context for the message after it.";
		body.messages.push(RequestMessage::developer(content.into()));

		let parsed = parse(message);
		body.messages.push(parsed);
	}

	let parsed = parse(message);
	body.messages.push(parsed);

	let response = request(body).await?;
	body.messages.push(RequestMessage::assistant(response.clone()));

	Ok(response)
}

pub async fn request(body: &RequestBody) -> Result<String> {
	let key = env::var("OPENAI_API_KEY")?;
	let url = env::var("OPENAI_API_URL")?;

	let client = Client::builder().user_agent(env!("CARGO_PKG_NAME")).build()?;
	let full = Url::parse(&url)?.join("chat/completions")?;

	let request = client.post(full).bearer_auth(key).json(body);
	let response: ResponseBody = request.send().await?.json().await?;

	let success = match response {
		ResponseBody::Error(error) => anyhow::bail!("API error: {}", error.error.message),
		ResponseBody::Success(success) => success,
	};

	let option = success.choices.into_iter().next();
	let choice = option.ok_or_else(|| Error::msg("No choices returned!"))?;

	Ok(choice.message.content)
}
