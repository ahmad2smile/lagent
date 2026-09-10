use anyhow::anyhow;
use tokio::process::Command;

pub(crate) async fn run_shell(command: &str) -> anyhow::Result<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(command).output()
    } else {
        Command::new("sh").arg("-c").arg(command).output()
    }
    .await
    .map_err(|e| anyhow!(format!("Failed to execute shell command: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let result = if output.status.success() {
        stdout
    } else {
        format!("Error:\nstdout:{}\nstderr:{}", stdout, stderr)
    };

    Ok(result)
}
