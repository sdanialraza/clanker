use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RequestBody {
	pub messages: Vec<RequestMessage>,
	pub model: String,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum RequestContent {
	ImageUrl(RequestContentImageUrl),
	Text(RequestContentText),
}

#[derive(Serialize)]
pub struct RequestContentImageUrl {
	pub image_url: RequestImageUrl,
}

#[derive(Serialize)]
pub struct RequestContentText {
	pub text: String,
}

#[derive(Serialize)]
pub struct RequestImageUrl {
	pub url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "role")]
pub enum RequestMessage {
	Assistant(RequestMessageAssistant),
	Developer(RequestMessageDeveloper),
	User(RequestMessageUser),
}

#[derive(Serialize)]
pub struct RequestMessageAssistant {
	pub content: String,
}

#[derive(Serialize)]
pub struct RequestMessageDeveloper {
	pub content: String,
}

#[derive(Serialize)]
pub struct RequestMessageUser {
	pub content: Vec<RequestContent>,
	pub name: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ResponseBody {
	Error(ResponseBodyError),
	Success(ResponseBodySuccess),
}

#[derive(Deserialize)]
pub struct ResponseBodyError {
	pub error: ResponseError,
}

#[derive(Deserialize)]
pub struct ResponseBodySuccess {
	pub choices: Vec<ResponseChoice>,
}

#[derive(Deserialize)]
pub struct ResponseChoice {
	pub message: ResponseMessage,
}

#[derive(Deserialize)]
pub struct ResponseError {
	pub message: String,
}

#[derive(Deserialize)]
pub struct ResponseMessage {
	pub content: String,
}

impl RequestContent {
	pub const fn image_url(image_url: RequestImageUrl) -> Self {
		Self::ImageUrl(RequestContentImageUrl { image_url })
	}

	pub const fn text(text: String) -> Self {
		Self::Text(RequestContentText { text })
	}
}

impl RequestMessage {
	pub const fn assistant(content: String) -> Self {
		Self::Assistant(RequestMessageAssistant { content })
	}

	pub const fn developer(content: String) -> Self {
		Self::Developer(RequestMessageDeveloper { content })
	}

	pub const fn user(content: Vec<RequestContent>, name: String) -> Self {
		Self::User(RequestMessageUser { content, name })
	}
}
