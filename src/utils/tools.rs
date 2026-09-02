use super::path_utils;
use rig::tool::ToolExecutionError;
use std::{fs, path::Path};

#[rig::tool_macro(
    description = "Read the contents of a file at the specified path. Returns the file content as a string.",
    required(path_string)
)]
pub async fn read_file(path_string: String) -> Result<String, ToolExecutionError> {
    let path = path_utils::to_abs_path(Path::new(&path_string))
        .map_err(|e| ToolExecutionError::other(format!("{e}")))?;

    path_utils::assert_cwd_permission(
        &path,
        "Not allowed to read outside of the current working directory.",
    )
    .map_err(|e| ToolExecutionError::other(format!("{e}")))?;

    std::fs::read_to_string(&path).map_err(|e| {
        ToolExecutionError::other(format!(
            "Failed to read file '{path}': {e}",
            path = path.display()
        ))
    })
}

#[rig::tool_macro(
    description = "Write content to a file at the specified path. Creates parent directories if they don't exist. Overwrites the file if it already exists.",
    required(path_string)
)]
pub async fn write_file(
    path_string: String,
    content: String,
) -> anyhow::Result<(), ToolExecutionError> {
    let path = &path_utils::to_abs_path(Path::new(&path_string))
        .map_err(|e| ToolExecutionError::other(format!("{e}")))?;

    path_utils::assert_cwd_permission(
        path,
        "Not allowed to write outside of the current working directory.",
    )
    .map_err(|e| ToolExecutionError::other(format!("{e}")))?;

    if !fs::exists(path).is_ok()
        && let Some(parent) = path.parent()
    {
        fs::create_dir(parent).map_err(|e| {
            ToolExecutionError::other(format!(
                "Failed to create dir '{path}': {e}",
                path = parent.display()
            ))
        })?;
    }

    fs::write(&path, &content).map_err(|e| {
        ToolExecutionError::other(format!(
            "Failed to write file '{path}': {e}",
            path = path.display()
        ))
    })
}
