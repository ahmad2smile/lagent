use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::bail;

pub(super) fn to_abs_path(path: &Path) -> anyhow::Result<PathBuf> {
    if Path::new(path).is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

pub(super) fn assert_cwd_permission(path: &Path, error: &str) -> anyhow::Result<()> {
    let cwd_path = env::current_dir()?;

    if let Some(path_str) = path.to_str()
        && path_str.contains("..")
    {
        bail!("path can not contain dir traverse sequence")
    }

    if !path.starts_with(cwd_path.clone()) {
        bail! {
            "{error}, request path: '{path}' is outside of {cwd_path}",
            path = path.display(),
            cwd_path = cwd_path.display()
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_cwd_permission_valid() {
        let path = env::current_dir().unwrap().join("./file.txt");

        assert_eq!(assert_cwd_permission(&path, "").is_err(), false);
    }

    #[test]
    fn test_assert_cwd_permission_invalid_root() {
        let path = Path::new("/root/user/file.txt");
        let pwd = env::current_dir().unwrap().into_string().unwrap();

        assert_eq!(
            assert_cwd_permission(&path, "").unwrap_err().to_string(),
            format!(", request path: '/root/user/file.txt' is outside of {pwd}")
        );
    }

    #[test]
    fn test_assert_cwd_permission_path_traverse() {
        let path = env::current_dir().unwrap().join("../file.txt");

        assert_eq!(
            assert_cwd_permission(&path, "").unwrap_err().to_string(),
            format!("path can not contain dir traverse sequence")
        );
    }
}
