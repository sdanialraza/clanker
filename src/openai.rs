use std::{env, fs};

use openai_api_rust::chat::{ChatApi, ChatBody};
use openai_api_rust::{Auth, Message, OpenAI, Role};

pub fn body() -> ChatBody {
	let message = Message {
		content: fs::read_to_string("assets/prompt.txt").unwrap(),
		role: Role::System,
	};

	ChatBody {
		frequency_penalty: None,
		logit_bias: None,
		max_tokens: Some(200),
		messages: vec![message],
		model: "gpt-5-chat-latest".into(),
		n: None,
		presence_penalty: None,
		stop: None,
		stream: None,
		temperature: None,
		top_p: None,
		user: None,
	}
}

pub fn post(body: &mut ChatBody, content: String, reply: Option<&str>) {
	let api_url = env::var("OPENAI_API_URL").unwrap();
	let auth = Auth::from_env().unwrap();
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

	let completion = openai.chat_completion_create(body).unwrap();
	let choice = completion.choices.into_iter().next().unwrap();
	let response = choice.message.unwrap();

	body.messages.push(Message {
		content: response.content,
		role: Role::Assistant,
	});
}
