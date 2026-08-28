use futures::StreamExt;
use rig::{
    Agent, client::AgentClientExt, completion::Prompt, message::Message, providers::openai,
    streaming::StreamingPrompt,
};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout},
    stream,
};

mod tools;

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
    print!("> ");
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

                let mut response = String::new();
                let mut input_tokens = 0u64;
                let mut output_tokens = 0u64;

                while let Some(chunk) = stream.next().await {}

                println!("");

                history.push(Message::user(input));

                if !response.is_empty() {
                    history.push(Message::assistant(response));
                }
            }

            return Ok(true);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = openai::Client::builder()
        .base_url("http://localhost:8080/v1".to_string())
        .api_key("no-key")
        .build()?;

    let agent = client
        .agent("LFM2.5-2.6B")
        .preamble(SYSTEM_PROMPT)
        .tool(tools::ReadFile)
        .tool(tools::WriteFile)
        .max_tokens(MAX_TOKENS)
        .build();

    let res = agent.prompt("What is Rust programming language?").await?;

    println!("{res}");

    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();

    let mut history: Vec<String> = vec![];

    loop {
        if !loop_handler(&mut stdin, &mut stdout, &mut history).await? {
            break;
        }
    }

    Ok(())
}
