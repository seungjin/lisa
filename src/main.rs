use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, Read};
use std::path::PathBuf;
use std::process;
use std::time::Duration;
use tiktoken_rs::*;
use url::Url;
use waki::Client;

const OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'k', long, env = "OPENAI_API_KEY")]
    key: String,
    #[arg(
        short,
        long,
        env = "OPENAI_MODEL_NAME",
        default_value = "gpt-4o-mini"
    )]
    model: String,
    #[arg(
        short,
        long,
        env = "OPENAI_PROMPT_TEXT",
        default_value = "You are a friendly assistant"
    )]
    prompt: String,
    #[arg(short, long, env = "OPENAI_TEMPERATURE", default_value = "0.75")]
    temp: f32,
    #[arg(short = 's', long, env = "OPENAI_MAX_TOKENS", default_value = "100")]
    max_tokens: usize,
    #[arg(trailing_var_arg = true)]
    input: Vec<String>,
}

#[derive(Debug)]
struct OpenaiAsk {
    api_key: String,
    model: String,
    prompt: String,
    temp: f32,
    max_tokens: usize,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
    max_tokens: usize,
}

#[derive(Debug, Default)]
struct OpenaiAskBuilder {}

impl OpenaiAskBuilder {
    pub fn new(
        api_key: String,
        model: String,
        prompt: String,
        temp: f32,
        max_tokens: usize,
        messages: Vec<Message>,
    ) -> OpenaiAsk {
        OpenaiAsk {
            api_key,
            model,
            prompt,
            temp,
            max_tokens,
            messages,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.input.join(" ").len() == 0 {
        println!("May I help you?");
        return Ok(());
    }

    let system_message = Message {
        role: "system".to_string(),
        content: cli.prompt.clone(),
    };
    let user_message = Message {
        role: "user".to_string(),
        content: cli.input.join(" "),
    };

    let oa = OpenaiAskBuilder::new(
        cli.key,
        cli.model,
        cli.prompt,
        cli.temp,
        cli.max_tokens,
        vec![system_message, user_message],
    );

    let aa = ask(oa).unwrap();

    let r = aa["choices"]
        .get(0)
        .unwrap()
        .get("message")
        .unwrap()
        .get("content")
        .unwrap()
        .as_str()
        .unwrap();

    println!("{}", r);
    Ok(())
}

fn ask(oa: OpenaiAsk) -> Result<Value> {
    let request_body = RequestBody {
        model: oa.model,
        messages: oa.messages,
        max_tokens: oa.max_tokens,
    };

    let resp = Client::new()
        .post(OPENAI_ENDPOINT)
        .json(&request_body)
        .connect_timeout(Duration::from_secs(5))
        .headers([
            ("Content-Type", "application/json"),
            ("Accept", "*/*"),
            ("Authorization", format!("Bearer {}", oa.api_key).as_str()),
        ])
        .send()?;

    resp.json()
}
