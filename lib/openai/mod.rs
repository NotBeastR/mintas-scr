use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use serde_json::{json, Value as JsonValue};
pub struct OpenAIModule;
impl OpenAIModule {
    pub fn call_function(function_name: &str, args: &[Value]) -> MintasResult<Value> {
        match function_name {
            "completion" => Self::completion(args),
            "chat_completion" => Self::chat_completion(args),
            "set_base_url" => Self::set_base_url(args),
            "get_models" => Self::get_models(args),
            "embedding" => Self::embedding(args),
            "image_generation" => Self::image_generation(args),
            "transcription" => Self::transcription(args),
            "moderation" => Self::moderation(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown function '{}' in openai module", function_name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn completion(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.completion".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let api_key = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "API Key must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let prompt = match &args[1] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "Prompt must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let model = if args.len() > 2 {
            match &args[2] {
                Value::String(s) => s.clone(),
                _ => "gpt-3.5-turbo-instruct".to_string(),
            }
        } else {
            "gpt-3.5-turbo-instruct".to_string()
        };
        let max_tokens = if args.len() > 3 {
            match &args[3] {
                Value::Number(n) => *n as u32,
                _ => 100,
            }
        } else {
            100
        };
        let temperature = if args.len() > 4 {
            match &args[4] {
                Value::Number(n) => *n as f32,
                _ => 0.7,
            }
        } else {
            0.7
        };
        let base_url = Self::get_base_url();
        let client = reqwest::blocking::Client::new();
        let request_body = json!({
            "model": model,
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature
        });
        match client.post(&format!("{}/v1/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let json_response: JsonValue = response.json().unwrap_or_default();
                        if let Some(choices) = json_response["choices"].as_array() {
                            if let Some(first_choice) = choices.first() {
                                if let Some(text) = first_choice["text"].as_str() {
                                    return Ok(Value::String(text.to_string()));
                                }
                            }
                        }
                        Ok(Value::String(json_response.to_string()))
                    } else {
                        let error_text = response.text().unwrap_or_default();
                        Err(MintasError::RuntimeError {
                            message: format!("OpenAI API error: {}", error_text),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("OpenAI request failed: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
    }
    fn chat_completion(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.chat_completion".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let api_key = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "API Key must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let messages = match &args[1] {
            Value::Array(arr) => {
                let mut msg_array = Vec::new();
                for msg in arr {
                    if let Value::Table(table) = msg {
                        let mut msg_obj = serde_json::Map::new();
                        for (key, value) in table {
                            if let Value::String(s) = value {
                                msg_obj.insert(key.clone(), JsonValue::String(s.clone()));
                            }
                        }
                        msg_array.push(JsonValue::Object(msg_obj));
                    }
                }
                msg_array
            }
            _ => return Err(MintasError::TypeError {
                message: "Messages must be an array of tables".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let model = if args.len() > 2 {
            match &args[2] {
                Value::String(s) => s.clone(),
                _ => "gpt-3.5-turbo".to_string(),
            }
        } else {
            "gpt-3.5-turbo".to_string()
        };
        let max_tokens = if args.len() > 3 {
            match &args[3] {
                Value::Number(n) => *n as u32,
                _ => 150,
            }
        } else {
            150
        };
        let temperature = if args.len() > 4 {
            match &args[4] {
                Value::Number(n) => *n as f32,
                _ => 0.7,
            }
        } else {
            0.7
        };
        let base_url = Self::get_base_url();
        let client = reqwest::blocking::Client::new();
        let request_body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": temperature
        });
        match client.post(&format!("{}/v1/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let json_response: JsonValue = response.json().unwrap_or_default();
                        if let Some(choices) = json_response["choices"].as_array() {
                            if let Some(first_choice) = choices.first() {
                                if let Some(message) = first_choice["message"].as_object() {
                                    if let Some(content) = message["content"].as_str() {
                                        return Ok(Value::String(content.to_string()));
                                    }
                                }
                            }
                        }
                        Ok(Value::String(json_response.to_string()))
                    } else {
                        let error_text = response.text().unwrap_or_default();
                        Err(MintasError::RuntimeError {
                            message: format!("OpenAI API error: {}", error_text),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("OpenAI request failed: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
    }
    fn set_base_url(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.set_base_url".to_string(),
                expected: 1,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let base_url = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "Base URL must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        std::env::set_var("MINTAS_OPENAI_BASE_URL", base_url);
        Ok(Value::String(format!("Base URL set to: {}", base_url)))
    }
    fn get_models(args: &[Value]) -> MintasResult<Value> {
        if args.is_empty() {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.get_models".to_string(),
                expected: 1,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let api_key = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "API Key must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let base_url = Self::get_base_url();
        let client = reqwest::blocking::Client::new();
        match client.get(&format!("{}/v1/models", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let json_response: JsonValue = response.json().unwrap_or_default();
                        Ok(Value::String(json_response.to_string()))
                    } else {
                        let error_text = response.text().unwrap_or_default();
                        Err(MintasError::RuntimeError {
                            message: format!("OpenAI API error: {}", error_text),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("OpenAI request failed: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
    }
    fn embedding(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.embedding".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let api_key = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "API Key must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let input = match &args[1] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "Input must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let model = if args.len() > 2 {
            match &args[2] {
                Value::String(s) => s.clone(),
                _ => "text-embedding-ada-002".to_string(),
            }
        } else {
            "text-embedding-ada-002".to_string()
        };
        let base_url = Self::get_base_url();
        let client = reqwest::blocking::Client::new();
        let request_body = json!({
            "model": model,
            "input": input
        });
        match client.post(&format!("{}/v1/embeddings", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let json_response: JsonValue = response.json().unwrap_or_default();
                        Ok(Value::String(json_response.to_string()))
                    } else {
                        let error_text = response.text().unwrap_or_default();
                        Err(MintasError::RuntimeError {
                            message: format!("OpenAI API error: {}", error_text),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("OpenAI request failed: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
    }
    fn image_generation(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.image_generation".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let api_key = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "API Key must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let prompt = match &args[1] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "Prompt must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let size = if args.len() > 2 {
            match &args[2] {
                Value::String(s) => s.clone(),
                _ => "1024x1024".to_string(),
            }
        } else {
            "1024x1024".to_string()
        };
        let n = if args.len() > 3 {
            match &args[3] {
                Value::Number(num) => *num as u32,
                _ => 1,
            }
        } else {
            1
        };
        let base_url = Self::get_base_url();
        let client = reqwest::blocking::Client::new();
        let request_body = json!({
            "prompt": prompt,
            "n": n,
            "size": size
        });
        match client.post(&format!("{}/v1/images/generations", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let json_response: JsonValue = response.json().unwrap_or_default();
                        Ok(Value::String(json_response.to_string()))
                    } else {
                        let error_text = response.text().unwrap_or_default();
                        Err(MintasError::RuntimeError {
                            message: format!("OpenAI API error: {}", error_text),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("OpenAI request failed: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
    }
    fn transcription(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.transcription".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let _api_key = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "API Key must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let file_path = match &args[1] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "File path must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let model = if args.len() > 2 {
            match &args[2] {
                Value::String(s) => s.clone(),
                _ => "whisper-1".to_string(),
            }
        } else {
            "whisper-1".to_string()
        };
        let base_url = Self::get_base_url();
        let response_text = format!("Transcription request for file: {} using model: {} (Base URL: {})", file_path, model, base_url);
        Ok(Value::String(response_text))
    }
    fn moderation(args: &[Value]) -> MintasResult<Value> {
        if args.len() < 2 {
            return Err(MintasError::InvalidArgumentCount {
                function: "openai.moderation".to_string(),
                expected: 2,
                got: args.len(),
                location: SourceLocation::new(0, 0),
            });
        }
        let api_key = match &args[0] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "API Key must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let input = match &args[1] {
            Value::String(s) => s,
            _ => return Err(MintasError::TypeError {
                message: "Input must be a string".to_string(),
                location: SourceLocation::new(0, 0),
            }),
        };
        let base_url = Self::get_base_url();
        let client = reqwest::blocking::Client::new();
        let request_body = json!({
            "input": input
        });
        match client.post(&format!("{}/v1/moderations", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let json_response: JsonValue = response.json().unwrap_or_default();
                        Ok(Value::String(json_response.to_string()))
                    } else {
                        let error_text = response.text().unwrap_or_default();
                        Err(MintasError::RuntimeError {
                            message: format!("OpenAI API error: {}", error_text),
                            location: SourceLocation::new(0, 0),
                        })
                    }
                }
                Err(e) => Err(MintasError::RuntimeError {
                    message: format!("OpenAI request failed: {}", e),
                    location: SourceLocation::new(0, 0),
                }),
            }
    }
    fn get_base_url() -> String {
        std::env::var("MINTAS_OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string())
    }
}