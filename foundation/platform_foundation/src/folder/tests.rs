use std::ffi::OsString;
use std::path::PathBuf;

use super::{absolute, platform};

/// 試験の環境の変数 `variables`（名前と値）だけを、絶対のパスとして読む
fn reader<'a>(variables: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<PathBuf> + 'a {
    move |name| {
        absolute(
            variables
                .iter()
                .find(|(variable, _)| *variable == name)
                .map(|(_, value)| OsString::from(value)),
        )
    }
}

/// 試験の絶対のパス（OS ごとの書き方）
fn absolute_path(path: &str) -> String {
    if cfg!(windows) {
        format!("C:\\{path}")
    } else {
        format!("/{path}")
    }
}

#[test]
fn an_empty_or_relative_value_is_not_used() {
    assert_eq!(absolute(None), None);
    assert_eq!(absolute(Some(OsString::from(""))), None);
    assert_eq!(absolute(Some(OsString::from("relative"))), None);
    assert_eq!(
        absolute(Some(OsString::from(absolute_path("folder")))),
        Some(PathBuf::from(absolute_path("folder")))
    );
}

#[cfg(windows)]
#[test]
fn the_folders_on_windows_come_from_localappdata_and_appdata() {
    let local = absolute_path("local");
    let roaming = absolute_path("roaming");
    let variables = [
        ("LOCALAPPDATA", local.as_str()),
        ("APPDATA", roaming.as_str()),
    ];
    assert_eq!(
        platform::data_folder(reader(&variables)),
        Some(PathBuf::from(&local))
    );
    assert_eq!(
        platform::config_folder(reader(&variables)),
        Some(PathBuf::from(&roaming))
    );

    // 相対のパスは使わない
    let variables = [("LOCALAPPDATA", "relative"), ("APPDATA", "relative")];
    assert_eq!(platform::data_folder(reader(&variables)), None);
    assert_eq!(platform::config_folder(reader(&variables)), None);
}

#[cfg(not(windows))]
#[test]
fn the_xdg_folders_are_used_and_otherwise_the_home_folder() {
    let variables = [
        ("XDG_DATA_HOME", "/data"),
        ("XDG_CONFIG_HOME", "/config"),
        ("HOME", "/home/user"),
    ];
    assert_eq!(
        platform::data_folder(reader(&variables)),
        Some(PathBuf::from("/data"))
    );
    assert_eq!(
        platform::config_folder(reader(&variables)),
        Some(PathBuf::from("/config"))
    );

    // XDG の変数が相対のパスなら、ホームのフォルダの下を使う
    let variables = [
        ("XDG_DATA_HOME", "relative"),
        ("XDG_CONFIG_HOME", "relative"),
        ("HOME", "/home/user"),
    ];
    assert_eq!(
        platform::data_folder(reader(&variables)),
        Some(PathBuf::from("/home/user/.local/share"))
    );
    assert_eq!(
        platform::config_folder(reader(&variables)),
        Some(PathBuf::from("/home/user/.config"))
    );

    // ホームのフォルダも分からなければ、分からない
    assert_eq!(platform::data_folder(reader(&[])), None);
    assert_eq!(
        platform::config_folder(reader(&[("HOME", "relative")])),
        None
    );
}

#[cfg(windows)]
#[test]
fn network_folders_are_not_on_this_pc() {
    // Windows のフォルダは、この PC のドライブにある（一時のフォルダは、ネットワークに置く環境があるため使わない）
    let system = std::fs::canonicalize(std::env::var_os("SystemRoot").unwrap()).unwrap();
    assert!(super::is_on_this_pc(&system));
    // UNC のパス（ネットワークのフォルダ）・ドライブのないパスは、この PC のフォルダとしない
    assert!(!super::is_on_this_pc(std::path::Path::new(
        r"\\server\share\folder"
    )));
    assert!(!super::is_on_this_pc(std::path::Path::new(r"relative")));
}
