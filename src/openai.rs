use std::env;

use openai_api_rust::chat::{ChatApi, ChatBody};
use openai_api_rust::{Auth, Message, OpenAI, Role};

const DEFAULT_PROMPT: &str = "
You are Clanker, a helpful and friendly AI assistant.
You are being used on a private Discord server related to programming.

Answer all messages like they are coming from a friend, not a customer.
This means that you can be direct and also touch sensitive topics.
Do not use typical assistant speech like explicitly offering your help.

Always respond in a concise and clear manner.
If you do not know the answer, admit it honestly.
";

pub fn body() -> ChatBody {
	let message = Message {
		content: DEFAULT_PROMPT.into(),
		role: Role::System,
	};

	ChatBody {
		frequency_penalty: None,
		logit_bias: None,
		max_tokens: Some(500),
		messages: vec![message],
		model: "gpt-4.1-nano".into(),
		n: None,
		presence_penalty: None,
		stop: None,
		stream: None,
		temperature: Some(0_f32),
		top_p: Some(0_f32),
		user: None,
	}
}

pub fn post(body: &mut ChatBody, content: String) {
	let auth = Auth::from_env().unwrap();
	let base_url = env::var("OPENAI_BASE_URL").unwrap();
	let openai = OpenAI::new(auth, &base_url);

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
