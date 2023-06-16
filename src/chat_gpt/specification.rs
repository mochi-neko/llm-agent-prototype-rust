use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Model {
    Gpt35Turbo,
    Gpt35Turbo0613,
    Gpt35Turbo16k,
    Gpt35Turbo16k0613,
}

impl Model {
    pub(super) fn parse_to_string(&self) -> Result<String> {
        match self {
            Model::Gpt35Turbo => Ok("gpt-3.5-turbo".to_string()),
            Model::Gpt35Turbo0613 => Ok("gpt-3.5-turbo-0613".to_string()),
            Model::Gpt35Turbo16k => Ok("gpt-3.5-turbo-16k".to_string()),
            Model::Gpt35Turbo16k0613 => Ok("gpt-3.5-turbo-16k-0613".to_string()),
            _ => Err(anyhow!("Invalid model")),
        }
    }

    pub(super) fn parse_to_model(input: &str) -> Result<Model> {
        match input {
            "gpt-3.5-turbo" => Ok(Model::Gpt35Turbo),
            "gpt-3.5-turbo-0613" => Ok(Model::Gpt35Turbo0613),
            "gpt-3.5-turbo-16k" => Ok(Model::Gpt35Turbo16k),
            "gpt-3.5-turbo-16k-0613" => Ok(Model::Gpt35Turbo16k0613),
            _ => Err(anyhow!("Invalid model")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) enum Role {
    System,
    Assistant,
    User,
    Function,
}

impl Role {
    pub(super) fn parse_to_string(&self) -> Result<String> {
        match self {
            Role::System => Ok("system".to_string()),
            Role::Assistant => Ok("assistant".to_string()),
            Role::User => Ok("user".to_string()),
            Role::Function => Ok("function".to_string()),
        }
    }

    pub(super) fn parse_to_role(input: &str) -> Result<Role> {
        match input {
            "system" => Ok(Role::System),
            "assistant" => Ok(Role::Assistant),
            "user" => Ok(Role::User),
            "function" => Ok(Role::Function),
            _ => Err(anyhow!("Invalid role")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct RequestBody {
    pub(super) model: String,
    pub(super) messages: Vec<Message>,
    // TODO: Add optional parameters
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct ResponseBody {
    pub(super) id: String,
    pub(super) object: String,
    pub(super) created: u64,
    pub(super) model: String,
    pub(super) choices: Vec<Choice>,
    pub(super) usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Message {
    pub(super) role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) function_call: Option<FunctionCall>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Choice {
    pub index: u64,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct FunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}
