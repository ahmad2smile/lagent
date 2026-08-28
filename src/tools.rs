use std::path::Path;

use rig::tool::ToolExecutionError;

#[rig::tool_macro(
    description = "Read the contents of a file at the specified path. Returns the file content as a string.",
    required(path)
)]
pub async fn read_file(path: String) -> Result<String, ToolExecutionError> {
    std::fs::read_to_string(&path).map_err(|e| {
        ToolExecutionError::new(
            rig::tool::ToolErrorKind::Other,
            format!("Failed to read file '{path}': {e}"),
        )
    })
}

#[rig::tool_macro(
    description = "Write content to a file at the specified path. Creates parent directories if they don't exist. Overwrites the file if it already exists.",
    required(path)
)]
pub async fn write_file(path: String, content: String) -> Result<(), ToolExecutionError> {
    if let Some(parent) = Path::new(&path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir(parent).map_err(|e| {
            ToolExecutionError::new(
                rig::tool::ToolErrorKind::Other,
                format!("Failed to create dir '{path}': {e}"),
            )
        })?;
    }

    std::fs::write(&path, &content).map_err(|e| {
        ToolExecutionError::new(
            rig::tool::ToolErrorKind::Other,
            format!("Failed to write file '{path}': {e}"),
        )
    })
}
