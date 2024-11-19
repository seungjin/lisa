use anyhow::{anyhow, Error, Result};
use clap::Parser;
use http::{HeaderMap, HeaderName, HeaderValue, Uri};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use std::io::Write as _;
use std::str::FromStr;
use wasi::http::outgoing_handler::{self, RequestOptions};
use wasi::http::types::{Fields, Method, OutgoingBody, Scheme};

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
        default_value = "You are a friendly assistant."
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

struct Lisa;

wasi::cli::command::export!(Lisa);

impl wasi::exports::cli::run::Guest for Lisa {
    fn run() -> Result<(), ()> {
        let mut stdout = wasi::cli::stdout::get_stdout();

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

        // println!("{}", r);
        //

        stdout.write_all(r.as_bytes()).unwrap();
        stdout.flush().unwrap();
        Ok(())
    }
}

fn ask(oa: OpenaiAsk) -> Result<Value> {
    let request_body = RequestBody {
        model: oa.model,
        messages: oa.messages,
        max_tokens: oa.max_tokens,
    };

    let body_str = serde_json::to_string(&request_body)?;
    let body2 = body_str.into_bytes();
    let clen = body2.len().to_string();

    let openai_api_uri = OPENAI_ENDPOINT.parse::<Uri>().unwrap();

    let fields = Fields::new();
    fields
        .append(
            &"Content-Type".to_string(),
            &"application/json".as_bytes().to_vec(),
        )
        .unwrap();
    fields
        .append(&"Accept".to_string(), &"*/*".as_bytes().to_vec())
        .unwrap();
    fields
        .append(&"Content-Length".to_string(), &clen.as_bytes().to_vec())
        .unwrap();
    fields
        .append(
            &"Authorization".to_string(),
            &format!("Bearer {}", oa.api_key).as_bytes().to_vec(),
        )
        .unwrap();
    fields
        .append(
            &"User-Agent".to_string(),
            &"wasm-wasip2".as_bytes().to_vec(),
        )
        .unwrap();

    let outgoing_request = outgoing_handler::OutgoingRequest::new(fields);
    outgoing_request.set_method(&Method::Post).unwrap();
    outgoing_request.set_scheme(Some(&Scheme::Https)).unwrap();
    outgoing_request
        .set_authority(Some(openai_api_uri.host().unwrap()))
        .unwrap();
    outgoing_request
        .set_path_with_query(Some(openai_api_uri.path()))
        .unwrap();

    let options = RequestOptions::new();
    // https://github.com/bytecodealliance/wasi-rs/blob/1fe2eeaf6b459c44765cd78bf7f75c9512310bf6/src/bindings.rs#L484
    // It is nano seconds. 10_000_000_000 is 10 sec.
    options
        .set_connect_timeout(Some(10_000_000_000u64))
        .map_err(|()| anyhow!("failed to set connect_timeout"))?;

    let outgoing_body = outgoing_request
        .body()
        .map_err(|_| anyhow!("outgoing request write failed"))?;

    {
        let request_body = outgoing_body
            .write()
            .map_err(|_| anyhow!("outgoing request write failed"))?;
        request_body.blocking_write_and_flush(&body2)?;
    }
    OutgoingBody::finish(outgoing_body, None)?;

    let future_response =
        outgoing_handler::handle(outgoing_request, Some(options)).unwrap();
    let incoming_response = match future_response.get() {
        Some(result) => {
            result.map_err(|()| anyhow!("response already taken"))?
        }
        None => {
            let pollable = future_response.subscribe();
            pollable.block();

            future_response
                .get()
                .expect("incoming response available")
                .map_err(|()| anyhow!("response already taken"))
                .unwrap()
        }
    }?;
    //drop(future_response);

    //let status_code = incoming_response.status();
    //println!("{:?}", status_code);

    let mut hm = HeaderMap::new();
    let fields = incoming_response.headers();
    for field in fields.entries() {
        hm.append(
            HeaderName::from_str(field.0.as_str()).unwrap(),
            HeaderValue::from_bytes(field.1.as_slice()).unwrap(),
        );
    }

    let mut body: Vec<u8> = Vec::new();
    let incoming_body = incoming_response.consume().unwrap();
    let input_stream = incoming_body.stream().unwrap();

    loop {
        let item = match input_stream.read(1024) {
            Ok(x) => x,
            Err(_) => break,
        };
        if item.is_empty() {
            break;
        }
        for i in item.into_iter() {
            body.push(i);
        }
    }

    Ok(serde_json::from_slice::<serde_json::Value>(&body)?)
}
