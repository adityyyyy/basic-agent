use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, fs, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    #[allow(unused_variables)]
    let response: Value = client
        .chat()
        .create_byot(json!({
                          "model": "anthropic/claude-haiku-4.5",
                          "messages": [
                              {
                                  "role": "user",
                                  "content": args.prompt
                              }
                          ],
                          "tools": [
        {
                              "type": "function",
                "function": {
                  "name": "Read",
                  "description": "Read and return the contents of a file",
                  "parameters": {
                    "type": "object",
                    "properties": {
                      "file_path": {
                        "type": "string",
                        "description": "The path to the file to read"
                      }
                    },
                    "required": ["file_path"]
                  }
                }
                          }],
                      }))
        .await?;

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");
    if let Some(tool_calls) = response["choices"][0]["message"]
        .get("tool_calls")
        .and_then(|v| v.as_array())
    {
        // Tool call exists → dispatch first one
        if let Some(tool_call) = tool_calls.first() {
            let function = tool_call
                .get("function")
                .and_then(|v| v.as_object())
                .ok_or("Invalid function format")?;

            let name = function
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing function name")?;

            let args_str = function
                .get("arguments")
                .and_then(|v| v.as_str())
                .ok_or("Missing arguments")?;

            dispatch_tool(name, args_str)?;
        }
    } else {
        // No tool_calls → print normal content
        if let Some(content) = response["choices"][0]["message"]
            .get("content")
            .and_then(|v| v.as_str())
        {
            println!("{}", content);
        }
    }

    Ok(())
}

fn dispatch_tool(name: &str, args: &str) -> Result<(), Box<dyn std::error::Error>> {
    if name == "Read" {
        let parsed: Value = serde_json::from_str(args)?;
        let file_path = parsed
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("file path is missing")?;
        let contents = fs::read_to_string(file_path)?;
        println!("{}", contents);
    }
    Ok(())
}
