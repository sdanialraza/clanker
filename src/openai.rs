use std::{env, fs};

use anyhow::{Error, Result};
use openai_api_rust::chat::{ChatApi, ChatBody};
use openai_api_rust::{Auth, Message, OpenAI, Role};

pub fn body() -> Result<ChatBody> {
	let message = Message {
		content: fs::read_to_string("assets/prompt.txt")?,
		role: Role::System,
	};

	let body = ChatBody {
		frequency_penalty: None,
		logit_bias: None,
		max_tokens: Some(200),
		messages: vec![message],
		model: "gpt-4.1-nano".into(),
		n: None,
		presence_penalty: None,
		stop: None,
		stream: None,
		temperature: None,
		top_p: None,
		user: None,
	};

	Ok(body)
}

pub fn post(body: &mut ChatBody, content: String, reply: Option<&str>) -> Result<String> {
	let api_url = env::var("OPENAI_API_URL")?;
	let auth = Auth::from_env().map_err(Error::msg)?;
	let openai = OpenAI::new(auth, &api_url);

	if let Some(value) = reply {
		body.messages.push(Message {
			content: format!("The next message will be a reply to this: {value}"),
			role: Role::System,
		});
	}

	body.messages.push(Message {
		content,
		role: Role::User,
	});

	let completion = openai.chat_completion_create(body).map_err(Error::msg)?;
	let message = completion.choices.into_iter().flat_map(|choice| choice.message).next();
	let response = message.ok_or_else(|| Error::msg("No choice contained a message"))?;

	body.messages.push(Message {
		content: response.content.clone(),
		role: Role::Assistant,
	});

	Ok(response.content)
}
