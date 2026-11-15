use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{RENAME_WAITS_MS, rename, rename_with_retry, replace_file};

/// 試験ごとの空のフォルダ（一時のフォルダの下）
struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "redring-platform-file-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// フォルダの中のファイルの名前（名前の順）
    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.0)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write_bytes(bytes: &[u8]) -> impl FnOnce(&mut std::fs::File) -> io::Result<()> + '_ {
    move |file| file.write_all(bytes)
}

fn replace(path: &Path, temporary: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_file(path, temporary, write_bytes(bytes))
}

#[test]
fn a_new_file_is_created_and_an_old_one_is_replaced() {
    let folder = Folder::new("replace");
    let (path, temporary) = (folder.join("a.json"), folder.join("a.json.tmp"));
    replace(&path, &temporary, b"first").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"first");
    replace(&path, &temporary, b"second").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second");
    assert_eq!(folder.names(), ["a.json"]);
}

#[test]
fn a_temporary_file_left_by_an_earlier_write_is_overwritten() {
    let folder = Folder::new("left");
    let (path, temporary) = (folder.join("a.json"), folder.join("a.json.tmp"));
    std::fs::write(&temporary, b"a longer partial write").unwrap();
    replace(&path, &temporary, b"new").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"new");
    assert_eq!(folder.names(), ["a.json"]);
}

#[test]
fn a_failed_write_keeps_the_old_file_and_removes_the_temporary_file() {
    let folder = Folder::new("failed-write");
    let (path, temporary) = (folder.join("a.json"), folder.join("a.json.tmp"));
    std::fs::write(&path, b"old").unwrap();
    let error = replace_file(&path, &temporary, |file| {
        file.write_all(b"half")?;
        Err(io::Error::other("cannot make the rest"))
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "cannot make the rest");
    assert_eq!(std::fs::read(&path).unwrap(), b"old");
    assert_eq!(folder.names(), ["a.json"]);
}

#[test]
fn a_failed_replace_keeps_the_target_and_removes_the_temporary_file_at_once() {
    let folder = Folder::new("failed-replace");
    // フォルダの名前のところは、ファイルで置き換えられない（試し直しても通らないため、待たない）
    let (path, temporary) = (folder.join("a"), folder.join("a.tmp"));
    std::fs::create_dir(&path).unwrap();
    std::fs::write(path.join("inside"), b"kept").unwrap();
    let started = std::time::Instant::now();
    assert!(replace(&path, &temporary, b"new").is_err());
    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(std::fs::read(path.join("inside")).unwrap(), b"kept");
    assert_eq!(folder.names(), ["a"]);
}

#[test]
fn a_folder_is_renamed() {
    let folder = Folder::new("rename-folder");
    let (from, to) = (folder.join("1.tmp"), folder.join("1"));
    std::fs::create_dir(&from).unwrap();
    std::fs::write(from.join("job.json"), b"{}").unwrap();
    rename(&from, &to).unwrap();
    assert_eq!(std::fs::read(to.join("job.json")).unwrap(), b"{}");
    assert_eq!(folder.names(), ["1"]);
}

#[test]
fn the_error_type_of_the_writer_is_returned() {
    #[derive(Debug, PartialEq)]
    enum WriteError {
        Io(io::ErrorKind),
        Invalid,
    }
    impl From<io::Error> for WriteError {
        fn from(error: io::Error) -> Self {
            Self::Io(error.kind())
        }
    }
    let folder = Folder::new("error-type");
    let (path, temporary) = (folder.join("a.bin"), folder.join("a.bin.tmp"));
    let result = replace_file(&path, &temporary, |_| Err(WriteError::Invalid));
    assert_eq!(result, Err(WriteError::Invalid));
    assert!(folder.names().is_empty());
}

#[test]
fn a_transient_refusal_is_retried_until_the_rename_succeeds() {
    let mut attempts = 0;
    let mut waits = Vec::new();
    let result = rename_with_retry(
        || {
            attempts += 1;
            if attempts < 4 {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            } else {
                Ok(())
            }
        },
        |_| true,
        |wait| waits.push(wait),
    );
    assert!(result.is_ok());
    assert_eq!(attempts, 4);
    assert_eq!(
        waits,
        [1, 2, 4].map(Duration::from_millis),
        "the waits double from 1 ms"
    );
}

#[test]
fn a_lasting_refusal_is_returned_after_about_two_seconds_of_waits() {
    let mut attempts = 0;
    let mut waited = Duration::ZERO;
    let result = rename_with_retry(
        || {
            attempts += 1;
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        },
        |_| true,
        |wait| waited += wait,
    );
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(attempts, RENAME_WAITS_MS.len() + 1);
    assert_eq!(waited, Duration::from_millis(2005));
}

#[test]
fn an_error_that_is_not_transient_is_returned_at_once() {
    let mut attempts = 0;
    let result = rename_with_retry(
        || {
            attempts += 1;
            Err(io::Error::from(io::ErrorKind::NotFound))
        },
        |error| error.kind() == io::ErrorKind::PermissionDenied,
        |_| panic!("an error that is not transient is not waited for"),
    );
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_eq!(attempts, 1);
}

#[cfg(windows)]
mod windows {
    use std::os::windows::fs::OpenOptionsExt;
    use std::time::{Duration, Instant};

    use super::{Folder, replace};

    #[test]
    fn a_reader_holding_the_target_for_a_while_delays_the_replace_without_failing_it() {
        let folder = Folder::new("held");
        let (path, temporary) = (folder.join("a.json"), folder.join("a.json.tmp"));
        std::fs::write(&path, b"old").unwrap();
        // 削除の共有を許さずに開く（ウイルス対策ソフトなどと同じ。開いている間、名前の変更で置き換えられない）
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x1) // FILE_SHARE_READ だけ
            .open(&path)
            .unwrap();
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            drop(held);
        });
        let started = Instant::now();
        replace(&path, &temporary, b"new").unwrap();
        release.join().unwrap();
        assert!(started.elapsed() >= Duration::from_millis(150));
        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        assert_eq!(folder.names(), ["a.json"]);
    }

    #[test]
    fn a_read_only_target_is_refused_at_once() {
        let folder = Folder::new("read-only");
        let (path, temporary) = (folder.join("a.json"), folder.join("a.json.tmp"));
        std::fs::write(&path, b"old").unwrap();
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&path, permissions.clone()).unwrap();
        let started = Instant::now();
        let result = replace(&path, &temporary, b"new");
        let elapsed = started.elapsed();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(&path, permissions).unwrap();
        assert!(result.is_err());
        assert!(
            elapsed < Duration::from_millis(500),
            "not retried: {elapsed:?}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"old");
        assert_eq!(folder.names(), ["a.json"]);
    }
}
