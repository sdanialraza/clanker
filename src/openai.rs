use std::env;
use std::process::Command;

use anyhow::{Error, Result};
use reqwest::{Client, Url};
use serenity::all::{Context, Message};

use crate::model::{RequestBody, RequestContent, RequestImageUrl, RequestMessage, ResponseBody};

pub fn body(ctx: &Context) -> Result<RequestBody> {
	let hash = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output();
	let url = Command::new("git").args(["remote", "get-url", "origin"]).output();

	let hash = match hash {
		Ok(output) if output.status.success() => String::from_utf8(output.stdout)?,
		Ok(output) => anyhow::bail!("Git Error: {}", String::from_utf8_lossy(&output.stderr).trim()),
		Err(_) => env::var("GIT_HASH").unwrap_or_else(|_| String::from("unknown")),
	};

	let url = match url {
		Ok(output) if output.status.success() => String::from_utf8(output.stdout)?,
		Ok(output) => anyhow::bail!("Git Error: {}", String::from_utf8_lossy(&output.stderr).trim()),
		Err(_) => env::var("GIT_URL").unwrap_or_else(|_| String::from("unknown")),
	};

	let content = include_str!("../assets/prompt.txt")
		.replace("$hash", hash.trim())
		.replace("$url", url.trim())
		.replace("$id", &ctx.cache.current_user().id.to_string())
		.replace("$name", &ctx.cache.current_user().name)
		.replace("$tag", &ctx.cache.current_user().tag());

	let messages = vec![RequestMessage::system(content)];
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
		content.push(RequestContent::text(format!(
			"{}: {}",
			message.author.name, message.content
		)));
	}

	for url in images {
		content.push(RequestContent::image_url(RequestImageUrl { url }));
	}

	RequestMessage::user(content, message.author.tag())
}

pub async fn post(body: &mut RequestBody, message: &Message, reply: Option<&Message>) -> Result<String> {
	if let Some(message) = reply {
		let content = "Use the following message as context for the message after it.";
		body.messages.push(RequestMessage::system(content.into()));

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
