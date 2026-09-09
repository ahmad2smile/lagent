use colored::Colorize;
use futures::StreamExt;
use rig::{
    Agent,
    agent::MultiTurnStreamItem,
    client::AgentClientExt,
    message::Message,
    providers::openai,
    streaming::{
        StreamedAssistantContent, StreamedUserContent, StreamingPrompt, ToolCallDeltaContent,
    },
};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};

use crate::{tools::commands::Commands, utils::executor_utils};

mod tools;
mod utils;

const MAX_TOKENS: u64 = 8192;
const SYSTEM_PROMPT: &str = r#"You are lagent, an interactive AI coding agent.

You have access to following tools:
- read_file: Read file contents
- write_file: Create or replace file contents
"#;

async fn loop_handler(
    agent: &Agent,
    stdin: &mut BufReader<Stdin>,
    stdout: &mut Stdout,
    history: &mut Vec<Message>,
) -> Result<bool, anyhow::Error> {
    stdout.write_all(b"> ").await?;
    stdout.flush().await?;

    let mut input = String::new();

    match stdin.read_line(&mut input).await? {
        0 => {
            println!("\nCiao");
            Ok(false)
        }
        _ => {
            let input = input.trim();
            let command = Commands::from(input);

            match command {
                Commands::Exit => {
                    println!("Ciao");
                    return Ok(false);
                }
                Commands::New => {
                    history.clear();
                    println!("-----------------------------------------");
                    println!("---------------New Session---------------");
                    println!("-----------------------------------------");
                    return Ok(true);
                }
                Commands::Run(command_str) => {
                    let result = match executor_utils::run_shell(command_str) {
                        Ok(res) => format!("{command_str}\n Result:\n {res}"),
                        Err(err) => format!("{command_str}\n Error:\n {err}"),
                    };

                    history.push(Message::user(result));
                }
                Commands::Help => println!("Run commands: !ls or Send message as normal chat"),
                Commands::None => println!("Unknown command"),
            };

            if !input.is_empty() {
                let mut stream = agent.stream_prompt(input).history(history.iter()).await;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(item) => match item {
                            MultiTurnStreamItem::StreamAssistantItem(streamed_assistant_item) => {
                                match streamed_assistant_item {
                                    StreamedAssistantContent::Text(text) => {
                                        print!("{}", text.text)
                                    }
                                    StreamedAssistantContent::ToolCall {
                                        tool_call,
                                        internal_call_id: _,
                                    } => {
                                        println!(
                                            "\nTool Started: {:?}",
                                            tool_call.function.name.bright_black()
                                        )
                                    }
                                    StreamedAssistantContent::ToolCallDelta {
                                        internal_call_id: _,
                                        content,
                                    } => match content {
                                        ToolCallDeltaContent::Name(name) => {
                                            print!("\nTool Delta: {}\n", name.bright_black())
                                        }
                                        ToolCallDeltaContent::Delta(delta) => {
                                            print!("{}", delta.bright_black())
                                        }
                                    },
                                    StreamedAssistantContent::Reasoning { reasoning, id: _ } => {
                                        print!("{}", reasoning.display_text().bright_black());
                                    }
                                    StreamedAssistantContent::ReasoningDelta {
                                        id: _,
                                        provider_id: _,
                                        reasoning,
                                    } => print!("{}", reasoning.bright_black()),
                                    StreamedAssistantContent::Final(stream_final) => println!(
                                        "\nTotal Tokens: {}",
                                        stream_final.usage.total_tokens
                                    ),
                                    StreamedAssistantContent::Unknown(unknown_payload) => {
                                        println!("\nUnknow payload: {:?}", unknown_payload)
                                    }
                                }
                            }
                            MultiTurnStreamItem::ToolExecutionCommitted {
                                tool_call,
                                internal_call_id: _,
                            } => println!(
                                "\nTool Commited: {}",
                                tool_call.function.name.bright_black()
                            ),
                            MultiTurnStreamItem::StreamUserItem(streamed_user_item) => {
                                match streamed_user_item {
                                    StreamedUserContent::ToolResult {
                                        tool_result,
                                        internal_call_id: _,
                                    } => {
                                        println!(
                                            "\nTool Success: {}",
                                            tool_result.name.bright_black()
                                        )
                                    }
                                }
                            }
                            MultiTurnStreamItem::CompletionCall(completion_call) => {
                                println!("\nCompletition: {}", completion_call.usage.total_tokens)
                            }
                            MultiTurnStreamItem::ModelTurnRetried { turn } => {
                                println!("\nModel turn retired: {turn}")
                            }
                            MultiTurnStreamItem::FinalResponse(prompt_response) => {
                                println!("\nFinal Res: {}", prompt_response.output)
                            }
                        },
                        Err(e) => {
                            println!("\nError: {}", e);
                        }
                    }
                }

                println!("");

                history.push(Message::user(input));
            }

            return Ok(true);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = openai::CompletionsClient::builder()
        .base_url("http://localhost:8080/v1/".to_string())
        .api_key("sk-no-key")
        .build()?;

    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut history: Vec<Message> = vec![];

    let agent = client
        .agent("Qwen3.8-9B")
        .preamble(SYSTEM_PROMPT)
        .tool(tools::filesystem::ReadFile)
        .tool(tools::filesystem::WriteFile)
        .tool(tools::filesystem::ListDir)
        .max_tokens(MAX_TOKENS)
        .build();

    loop {
        if !loop_handler(&agent, &mut stdin, &mut stdout, &mut history).await? {
            break;
        }
    }

    Ok(())
}
