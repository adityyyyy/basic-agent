use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, fs, process};
use std::process::Command;

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

    let tools = json!([
    {
      "type": "function",
      "function": {
        "name": "Bash",
        "description": "Execute a shell command",
        "parameters": {
          "type": "object",
          "required": ["command"],
          "properties": {
            "command": {
              "type": "string",
              "description": "The command to execute"
            }
          }
        }
      }
    },
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
                },
        {
        "type": "function",
          "function": {
            "name": "Write",
            "description": "Write content to a file",
            "parameters": {
              "type": "object",
              "required": ["file_path", "content"],
              "properties": {
                "file_path": {
                  "type": "string",
                  "description": "The path of the file to write to"
                },
                "content": {
                  "type": "string",
                  "description": "The content to write to the file"
                }
              }
            }
          }
        }
            ]);

    let mut messages = vec![json!({
        "role": "user",
        "content": args.prompt
    })];

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "model": "anthropic/claude-haiku-4.5",
                "messages": messages,
                "tools": tools,
            }))
            .await?;

        eprintln!("Logs from your program will appear here!");

        let assistant_message = response["choices"][0]["message"].clone();
        messages.push(assistant_message.clone());

        let tool_calls = assistant_message
            .get("tool_calls")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if tool_calls.is_empty() {
            // No tool calls — print final content and exit
            if let Some(content) = assistant_message.get("content").and_then(|v| v.as_str()) {
                println!("{}", content);
            }
            break;
        }

        // Execute each tool call and add results to messages
        for tool_call in &tool_calls {
            let id = tool_call.get("id").and_then(|v| v.as_str()).unwrap_or("");

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

            let result = dispatch_tool(name, args_str)?;

            messages.push(json!({
                "role": "tool",
                "tool_call_id": id,
                "content": result,
            }));
        }
    }

    Ok(())
}

fn dispatch_tool(name: &str, args: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parsed: Value = serde_json::from_str(args)?;
    match name {
        "Read" => {
            let file_path = parsed
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("file path is missing")?;
            let contents = fs::read_to_string(file_path)?;
            Ok(contents)
        }
        "Write" => {
            let file_path = parsed
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("file path is missing")?;
            let contents = parsed
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("content is missing")?;
            fs::write(file_path, contents)?;
            Ok("File written successfully".to_string())
        }
        "Bash" => {
            let command = parsed
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or("command is missing")?;
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let result = format!("{}{}" , stdout, stderr);
            Ok(result)
        }
        _ => Err(format!("Unknown tool: {}", name).into()),
    }
}
