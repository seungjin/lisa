use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::time::Duration;
use waki::Client;

const TURSO_URL: &str = "https://log-seungjin.turso.io/v2/pipeline";

#[derive(Debug, Serialize, Deserialize)]
struct Body {
    requests: Vec<Request>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    #[serde(rename = "type")]
    type_: RequestType,
    #[serde(skip_serializing_if = "Option::is_none")]
    stmt: Option<Statement>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum RequestType {
    Execute,
    Close,
}

#[derive(Debug, Serialize, Deserialize)]
struct Statement {
    sql: String,
}

/*
{
  "requests": [
    { "type": "execute", "stmt": { "sql": "SELECT * FROM users" } },
    { "type": "close" }
  ]
}
*/

fn run(quries: Vec<String>) -> Result<i32> {
    let api_key = match env::var("TURSO_API_KEY") {
        Ok(v) => v,
        Err(e) => {
            println!("Can't find TURSO_API_KEY.");
            return Ok(-1);
        }
    };

    let mut body = Body {
        requests: Vec::new(),
    };
    for q in quries {
        let statement = Statement { sql: q };
        let a = Request {
            type_: RequestType::Execute,
            stmt: Some(statement),
        };
        body.requests.push(a);
    }

    body.requests.push(Request {
        type_: RequestType::Close,
        stmt: None,
    });

    let v = serde_json::to_value(&body).unwrap();

    println!("{:?}", v);

    let resp = Client::new()
        .post(TURSO_URL)
        //.json(&v)
        .connect_timeout(Duration::from_secs(5))
        .headers([
            ("Content-Type", "application/json"),
            ("Accept", "*/*"),
            ("Authorization", format!("Bearer {}", api_key).as_str()),
        ])
        .send()?;

    println!("{:?}", resp.body().unwrap());
    Ok(1)
}

#[test]
fn test_run() {
    let a = vec![
        "INSERT INTO openai_communication(body) VALUES(\"ars1\")".to_string(),
        "INSERT INTO openai_communication(body) VALUES(\"ars2\")".to_string(),
        "INSERT INTO openai_communication(body) VALUES(\"ars3\")".to_string(),
    ];
    let b = run(a).unwrap();

    println!("---{}", b);
}
