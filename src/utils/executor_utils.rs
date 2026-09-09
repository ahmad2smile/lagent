use anyhow::anyhow;

pub(crate) fn run_shell(command: &str) -> anyhow::Result<String> {
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| anyhow!(format!("Failed to execute shell command: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let result = if output.status.success() {
        stdout
    } else {
        format!("Error: {}\n{}", stdout, stderr)
    };

    Ok(result)
}
