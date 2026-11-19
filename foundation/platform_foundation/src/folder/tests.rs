use std::ffi::OsString;
use std::path::PathBuf;

use super::{
    PrivateFolderError, absolute, check_private_folder, create_private_folder, make_private,
    platform,
};

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
    assert!(platform::is_on_this_pc(&system));
    // UNC のパス（ネットワークのフォルダ）・ドライブのないパスは、この PC のフォルダとしない
    let unc = std::path::Path::new(r"\\server\share\folder");
    assert!(!platform::is_on_this_pc(unc));
    assert!(!platform::is_on_this_pc(std::path::Path::new(r"relative")));
    // UNC のパスのフォルダは、利用者だけが読み書きできるこの PC のフォルダとしない（フォルダを読みに行かない）
    assert!(matches!(
        platform::check_private_folder(unc),
        Err(PrivateFolderError::NotOnThisPc)
    ));
}

/// 試験の一時のフォルダ（終わったら消す）
struct TestFolder(PathBuf);

impl TestFolder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "redring-platform-folder-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }
}

impl Drop for TestFolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_created_private_folder_is_private_and_a_missing_folder_cannot_be_checked() {
    let root = TestFolder::new("created");
    let folder = root.0.join("parent").join("store");
    assert!(matches!(
        check_private_folder(&folder),
        Err(PrivateFolderError::Io(_))
    ));
    create_private_folder(&folder).unwrap();
    assert!(folder.is_dir());
    check_private_folder(&folder).unwrap();
    // すでにあれば何もしない
    create_private_folder(&folder).unwrap();
    // ファイルは、フォルダとしない
    let file = folder.join("file.txt");
    std::fs::write(&file, b"file").unwrap();
    assert!(matches!(
        check_private_folder(&file),
        Err(PrivateFolderError::Io(_))
    ));
}

#[cfg(windows)]
#[test]
fn a_folder_other_users_can_read_or_write_is_not_private_until_made_private() {
    let root = TestFolder::new("open");
    let folder = root.0.join("store");
    create_private_folder(&folder).unwrap();
    // Users に読み書き・Everyone に読むこと・Authenticated Users に中のものの書き込みを渡したフォルダは、利用者だけの
    // フォルダでない（中のものにだけ引き継ぐ権利も数える）
    for sddl in [
        "D:P(A;OICI;FA;;;{user})(A;OICI;0x1301bf;;;BU)",
        "D:P(A;OICI;FA;;;{user})(A;OICI;FR;;;WD)",
        "D:P(A;OICI;FA;;;{user})(A;OICIIO;FW;;;AU)",
    ] {
        platform::set_test_dacl(&folder, sddl).unwrap();
        assert!(
            matches!(
                check_private_folder(&folder),
                Err(PrivateFolderError::OpenToOthers)
            ),
            "{sddl}"
        );
    }
    // たどる・属性を読むだけの権利、拒む権利、SYSTEM・Administrators の権利は、ほかの利用者に渡していないとみなす
    platform::set_test_dacl(
        &folder,
        "D:P(A;;FA;;;{user})(A;;0x1200a0;;;WD)(D;;FA;;;AN)(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)",
    )
    .unwrap();
    check_private_folder(&folder).unwrap();
    // 利用者だけが読み書きできるようにする
    platform::set_test_dacl(&folder, "D:P(A;OICI;FA;;;{user})(A;OICI;FA;;;WD)").unwrap();
    make_private(&folder).unwrap();
    check_private_folder(&folder).unwrap();
}

#[cfg(unix)]
#[test]
fn a_folder_other_users_can_read_or_write_is_not_private_until_made_private() {
    use std::os::unix::fs::PermissionsExt;

    let root = TestFolder::new("open");
    let folder = root.0.join("store");
    create_private_folder(&folder).unwrap();
    let mode = std::fs::metadata(&folder).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
    for mode in [0o750, 0o705, 0o770, 0o755] {
        std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(mode)).unwrap();
        assert!(
            matches!(
                check_private_folder(&folder),
                Err(PrivateFolderError::OpenToOthers)
            ),
            "{mode:o}"
        );
    }
    make_private(&folder).unwrap();
    check_private_folder(&folder).unwrap();
}
