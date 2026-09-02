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

            if input.eq_ignore_ascii_case("exit")
                || input.eq_ignore_ascii_case("quit")
                || input.eq_ignore_ascii_case("/q")
            {
                println!("Ciao");
                return Ok(false);
            }

            if !input.is_empty() {
                let mut stream = agent
                    .stream_prompt(input)
                    .history(history.iter())
                    .max_turns(100)
                    .await;

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
                                        println!("\nTool call: {:?}", tool_call.signature)
                                    }
                                    StreamedAssistantContent::ToolCallDelta {
                                        internal_call_id: _,
                                        content,
                                    } => match content {
                                        ToolCallDeltaContent::Name(name) => {
                                            print!("{name}")
                                        }
                                        ToolCallDeltaContent::Delta(delta) => {
                                            print!("{delta}")
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
                                tool_call: _,
                                internal_call_id,
                            } => println!("\n{tool_call}", tool_call = internal_call_id),
                            MultiTurnStreamItem::StreamUserItem(streamed_user_item) => {
                                match streamed_user_item {
                                    StreamedUserContent::ToolResult {
                                        tool_result,
                                        internal_call_id: _,
                                    } => {
                                        println!("\n{:?}", tool_result)
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
        .tool(utils::tools::ReadFile)
        .tool(utils::tools::WriteFile)
        .max_tokens(MAX_TOKENS)
        .build();

    loop {
        if !loop_handler(&agent, &mut stdin, &mut stdout, &mut history).await? {
            break;
        }
    }

    Ok(())
}
