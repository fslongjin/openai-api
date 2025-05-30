// Given a chat conversation, the model will return a chat completion response.
// See: https://platform.openai.com/docs/api-reference/chat

//! Chat API

use std::collections::HashMap;

use crate::requests::Requests;
use crate::*;
use serde::{Deserialize, Serialize, Serializer};

fn serialize_f32_two_decimals<S>(value: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serializer.serialize_f64((*v as f64 * 100.0).round() / 100.0),
        None => serializer.serialize_none(),
    }
}


use super::{completions::Completion, CHAT_COMPLETION_CREATE};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatBody {
	/// ID of the model to use.
	/// See the model endpoint compatibility table for details on which models work with the Chat API.
	pub model: String,
	/// The messages to generate chat completions for, in the chat format.
	pub messages: Vec<Message>,
	/// What sampling temperature to use, between 0 and 2.
	/// Higher values like 0.8 will make the output more random,
	/// while lower values like 0.2 will make it more focused and deterministic.
	/// We generally recommend altering this or top_p but not both.
	/// Defaults to 1
	#[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_f32_two_decimals")]
	pub temperature: Option<f32>,
	/// An alternative to sampling with temperature, called nucleus sampling,
	/// where the model considers the results of the tokens with top_p probability mass.
	/// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
	/// We generally recommend altering this or temperature but not both.
	/// Defaults to 1
	#[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_f32_two_decimals")]
	pub top_p: Option<f32>,
	/// How many chat completion choices to generate for each input message.
	/// Defaults to 1
	#[serde(skip_serializing_if = "Option::is_none")]
	pub n: Option<i32>,
	/// If set, partial message deltas will be sent, like in ChatGPT.
	/// Tokens will be sent as data-only server-sent events as they become available,
	/// with the stream terminated by a data: [DONE] message. See the OpenAI Cookbook for example code.
	/// Defaults to false
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stream: Option<bool>,
	/// Up to 4 sequences where the API will stop generating further tokens.
	/// Defaults to null
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stop: Option<Vec<String>>,
	/// The maximum number of tokens to generate in the chat completion.
	/// The total length of input tokens and generated tokens is limited by the model's context length.
	/// Defaults to inf
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max_tokens: Option<i32>,
	/// Number between -2.0 and 2.0.
	/// Positive values penalize new tokens based on whether they appear in the text so far,
	/// increasing the model's likelihood to talk about new topics.
	/// Defaults to 0
	#[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_f32_two_decimals")]
	pub presence_penalty: Option<f32>,
	/// Number between -2.0 and 2.0.
	/// Positive values penalize new tokens based on their existing frequency in the text so far,
	/// decreasing the model's likelihood to repeat the same line verbatim.
	/// Defaults to 0
	#[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_f32_two_decimals")]
	pub frequency_penalty: Option<f32>,
	/// Modify the likelihood of specified tokens appearing in the completion.
	/// Accepts a json object that maps tokens (specified by their token ID in the tokenizer)
	/// to an associated bias value from -100 to 100. Mathematically,
	/// the bias is added to the logits generated by the model prior to sampling.
	/// The exact effect will vary per model, but values between -1 and 1 should
	/// decrease or increase likelihood of selection;
	/// values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
	/// Defaults to null
	#[serde(skip_serializing_if = "Option::is_none")]
	pub logit_bias: Option<HashMap<String, String>>,
	/// A unique identifier representing your end-user,
	/// which can help OpenAI to monitor and detect abuse. Learn more.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub user: Option<String>,
}

pub trait ChatApi {
	/// Creates a completion for the chat message
	fn chat_completion_create(&self, chat_body: &ChatBody) -> ApiResult<Completion>;
}

impl ChatApi for OpenAI {
	fn chat_completion_create(&self, chat_body: &ChatBody) -> ApiResult<Completion> {
		let request_body = serde_json::to_value(chat_body).unwrap();
		let res = self.post(CHAT_COMPLETION_CREATE, request_body)?;
		let completion: Completion = serde_json::from_value(res.clone()).unwrap();
		Ok(completion)
	}
}

#[cfg(test)]
mod tests {
	use crate::{apis::chat::ChatBody, openai::new_test_openai, Message, Role};

	use super::ChatApi;

	#[test]
	fn test_chat_completion() {
		let openai = new_test_openai();
		let body = ChatBody {
			model: "gpt-3.5-turbo".to_string(),
			max_tokens: Some(7),
			temperature: Some(0_f32),
			top_p: Some(0_f32),
			n: Some(2),
			stream: Some(false),
			stop: None,
			presence_penalty: None,
			frequency_penalty: None,
			logit_bias: None,
			user: None,
			messages: vec![Message { role: Role::User, content: "Hello!".to_string() }],
		};
		let rs = openai.chat_completion_create(&body);
		let choice = rs.unwrap().choices;
		let message = &choice[0].message.as_ref().unwrap();
		assert!(message.content.contains("Hello"));
	}
}
