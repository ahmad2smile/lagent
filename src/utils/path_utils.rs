use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::{Component, Path, PathBuf},
};

use anyhow::{anyhow, bail};

pub(super) fn to_abs_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

pub(super) fn assert_cwd_permission(path: &Path, error: &str) -> anyhow::Result<()> {
    let cwd_path = env::current_dir()?;

    if path.components().any(|c| c == Component::ParentDir) {
        bail!("path can not contain dir traverse sequence")
    }

    if !path.starts_with(&cwd_path) {
        bail! {
            "{error}, request path: '{path}' is outside of {cwd_path}",
            path = path.display(),
            cwd_path = &cwd_path.display()
        }
    }

    Ok(())
}

pub(super) fn read_file(
    path: &Path,
    lines_to_skip: Option<usize>,
    lines_to_read: Option<usize>,
) -> anyhow::Result<String> {
    let mut file_reader =
        BufReader::new(fs::File::open(path).map_err(|err| anyhow!("Unable to open file: {err}"))?);

    if Some(0) == lines_to_read {
        return Ok("".to_string());
    }

    let mut result = Vec::new();
    let mut line = Vec::new();
    let mut lines_to_skip = lines_to_skip.unwrap_or(0);
    let mut lines_to_read = lines_to_read.unwrap_or(usize::MAX);

    while lines_to_skip > 0 && file_reader.skip_until(b'\n')? > 0 {
        lines_to_skip -= 1;
    }

    loop {
        line.clear();
        if file_reader.read_until(b'\n', &mut line)? == 0 {
            break;
        }

        result.extend_from_slice(&line);
        lines_to_read -= 1;

        if lines_to_read == 0 {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&result).into_owned())
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

    const FILE_CONTENT: &str = "line1\nline2\nline3\nline4\nline5\n";
    const TEST_CASES: &[(Option<usize>, Option<usize>, &str, &str, &str)] = &[
        (None, None, FILE_CONTENT, FILE_CONTENT, "Read all lines"),
        (None, None, "a\nb", "a\nb", "No trailing newline"),
        (Some(1), None, "a\nb", "b", "Skip 1, no trailing newline"),
        (None, Some(1), "a\nb", "a\n", "Read 1, keeps its newline"),
        (
            Some(1),
            None,
            "single line of text",
            "",
            "Single line, skip past it",
        ),
        (
            None,
            Some(3),
            FILE_CONTENT,
            "line1\nline2\nline3\n",
            "Read first n lines",
        ),
        (
            Some(2),
            None,
            FILE_CONTENT,
            "line3\nline4\nline5\n",
            "Skip first n lines",
        ),
        (None, None, "", "", "Empty file"),
        (Some(100), None, FILE_CONTENT, "", "Read from EOF"),
        (
            None,
            Some(100),
            FILE_CONTENT,
            FILE_CONTENT,
            "Read beyond EOF",
        ),
        (
            None,
            None,
            "single line of text",
            "single line of text",
            "Single line no newline",
        ),
        (
            Some(1),
            None,
            "single line of text",
            "",
            "Single line no newline with Offset 1",
        ),
        (
            None,
            None,
            "single line of text\n",
            "single line of text\n",
            "Single line with newline",
        ),
        (Some(0), Some(0), FILE_CONTENT, "", "Skip 0 and read 0"),
        (
            Some(0),
            Some(1),
            FILE_CONTENT,
            "line1\n",
            "Skip 0 and read 1",
        ),
        (None, Some(0), FILE_CONTENT, "", "Read 0 lines"),
        (
            Some(5),
            Some(3),
            FILE_CONTENT,
            "",
            "Skip more than lines, read less than remaining",
        ),
        (
            Some(2),
            Some(10),
            FILE_CONTENT,
            "line3\nline4\nline5\n",
            "Skip 2, read 10 (more than remaining)",
        ),
        (
            Some(4),
            Some(1),
            FILE_CONTENT,
            "line5\n",
            "Skip 4, read 1 (last line)",
        ),
        (
            Some(4),
            Some(2),
            FILE_CONTENT,
            "line5\n",
            "Skip 4, read 2 (clamped to 1)",
        ),
        (
            Some(u32::MAX as usize),
            Some(1),
            FILE_CONTENT,
            "",
            "Skip u32::MAX",
        ),
        (
            Some(0),
            Some(u32::MAX as usize),
            FILE_CONTENT,
            FILE_CONTENT,
            "Read u32::MAX",
        ),
    ];

    #[test]
    fn test_read_file_parametrized() {
        let test_dir = env::current_dir().unwrap().join("target/lagent_test");

        if !test_dir.exists() {
            fs::create_dir_all(&test_dir).unwrap();
        }

        let test_file = test_dir.join("test_file.txt");

        for (lines_to_skip, lines_to_read, content, expected, description) in TEST_CASES {
            fs::write(&test_file, content).unwrap();
            let result = read_file(&test_file, *lines_to_skip, *lines_to_read).unwrap();
            assert_eq!(&result, expected, "Failed for case: {description}");
        }

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    #[should_panic(expected = "Unable to open file: No such file or directory (os error 2)")]
    fn test_read_file_invalid() {
        _ = read_file(Path::new("./invalid/file.txt"), None, None).unwrap();
    }
}
