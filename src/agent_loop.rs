use colored::Colorize;
use futures::StreamExt;
use rig::{
    Agent,
    agent::MultiTurnStreamItem,
    message::Message,
    streaming::{
        StreamedAssistantContent, StreamedUserContent, StreamingPrompt, ToolCallDeltaContent,
    },
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};

use crate::{tools::commands::Commands, utils::executor_utils};

pub(crate) async fn loop_handler(
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

            if input.is_empty() {
                return Ok(true);
            }

            match Commands::from(input) {
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
                Commands::Run(command) => {
                    let result = match executor_utils::run_shell(command).await {
                        Ok(res) => format!("{command}\nResult:\n{res}"),
                        Err(err) => format!("{command}\nError:\n{err}"),
                    };

                    println!("{result}");
                    history.push(Message::user(result));
                }
                Commands::Help => println!("Run commands: !ls or Send message as normal chat"),
                Commands::None => println!("Unknown command"),
            };

            let mut stream = agent
                .stream_prompt(input)
                .max_turns(100)
                .history(history.iter())
                .await;

            history.push(Message::user(input));

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(item) => match item {
                        MultiTurnStreamItem::StreamAssistantItem(streamed_assistant_item) => {
                            match streamed_assistant_item {
                                StreamedAssistantContent::Text(text) => {
                                    print!("{}", text.text)
                                }
                                StreamedAssistantContent::ToolCall {
                                    tool_call: _,
                                    internal_call_id: _,
                                } => {}
                                StreamedAssistantContent::ToolCallDelta {
                                    internal_call_id: _,
                                    content,
                                } => match content {
                                    ToolCallDeltaContent::Name(name) => {
                                        print!("\n{}\n", name.bright_black())
                                    }
                                    ToolCallDeltaContent::Delta(delta) => {
                                        print!("{}", delta.bright_black())
                                    }
                                },
                                StreamedAssistantContent::Reasoning { reasoning, id: _ } => {
                                    print!(
                                        "Reasoning: {}",
                                        reasoning.display_text().bright_black()
                                    );
                                }
                                StreamedAssistantContent::ReasoningDelta {
                                    id: _,
                                    provider_id: _,
                                    reasoning,
                                } => print!("{}", reasoning.bright_black()),
                                StreamedAssistantContent::Final(stream_final) => {
                                    println!("\nTotal Tokens: {}", stream_final.usage.total_tokens)
                                }
                                StreamedAssistantContent::Unknown(unknown_payload) => {
                                    println!("\nUnknow payload: {:?}", unknown_payload)
                                }
                            }
                        }
                        MultiTurnStreamItem::ToolExecutionCommitted {
                            tool_call: _,
                            internal_call_id: _,
                        } => {}
                        MultiTurnStreamItem::StreamUserItem(streamed_user_item) => {
                            match streamed_user_item {
                                StreamedUserContent::ToolResult {
                                    tool_result,
                                    internal_call_id: _,
                                } => {
                                    println!("\nTool Call: {}", tool_result.name.bright_black())
                                }
                            }
                        }
                        MultiTurnStreamItem::CompletionCall(completion_call) => {
                            let reson = match completion_call.finish_reason {
                                Some(reason) => format!("{:?}", reason),
                                None => "No Reason Found".to_string(),
                            };
                            println!("\nCompletion: {}", reson.bright_black())
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

            return Ok(true);
        }
    }
}
