//! 保存先の変更（選んだフォルダを確かめ、設定に書き、ジョブマネージャーを起動し直す）
//!
//! 保存先は、ジョブマネージャーのプロセスが起動のときに開く。替えるときは、選んだフォルダを保存先として開けるかを
//! 確かめてから設定に書き、新しいジョブマネージャーを起動し、このプロセスを終える。待ち・計算中のジョブ、取り込みを
//! 待つ投入がある間は替えない（今の保存先に残り、計算されなくなるため）。
//!
//! 保存先は、利用者だけが読み書きできるこの PC のフォルダにする（ほかの利用者が、ジョブの入力を読み、投入・成果物を
//! 書き換えられないように）。替えるときと起動のときに確かめる。

use std::convert::Infallible;
use std::path::Path;
use std::sync::mpsc::Sender;

use job_runtime::store::{JobStore, StoreError};
use platform_foundation::folder::PrivateFolderError;

use crate::service::ServiceRequest;

/// 選んだフォルダの中身
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FolderContents {
    /// 空のフォルダ
    Empty,
    /// ジョブの保存先（そのジョブを一覧に出す）
    Store,
    /// 保存先でないファイル・フォルダがある
    Other,
}

/// 保存先を替えられない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeFailure {
    /// 今の保存先と同じフォルダ
    Same,
    /// 今の保存先の中のフォルダ、または今の保存先を含むフォルダ
    Nested,
    /// この PC のフォルダでない（ネットワークのフォルダ）
    NotLocal,
    /// 利用者ごとの権限を付けられないファイルシステム（FAT32・exFAT など）のフォルダ
    NoPermissions,
    /// ほかの利用者も読み書きできるフォルダ（持ち主がほかの利用者のフォルダを含む）
    OpenToOthers,
    /// フォルダを読み書きできない
    Inaccessible,
    /// 中のフォルダの名前が、保存先の中のフォルダと重なる（ほかの保存先を含むフォルダなど）
    NameConflict,
    /// 待ち・計算中のジョブ、または取り込みを待つ投入がある
    Busy,
    /// 設定を保存できない
    SettingNotSaved,
    /// 設定は保存したが、ジョブマネージャーを起動し直せない
    RestartFailed,
}

/// 今の保存先 `current` を、フォルダ `chosen` に替えられるかを確かめ、その中身を返す
///
/// # Errors
///
/// 同じフォルダ・入れ子のフォルダ・利用者だけが読み書きできるこの PC のフォルダでない（ネットワークのフォルダ・
/// 権限を付けられないドライブ・ほかの利用者も読み書きできるフォルダ）・読めないフォルダ・中のフォルダの名前が
/// 保存先と重なるフォルダの場合。
pub(crate) fn check_folder(current: &Path, chosen: &Path) -> Result<FolderContents, ChangeFailure> {
    let chosen_real = std::fs::canonicalize(chosen).map_err(|_| ChangeFailure::Inaccessible)?;
    // 書いたとおりのパスでも比べる（今の保存先を読めない（外したドライブなど）ときは、実際のパスにできないため）
    let mut pairs = vec![(chosen.to_path_buf(), current.to_path_buf())];
    if let Ok(current_real) = std::fs::canonicalize(current) {
        pairs.push((chosen_real, current_real));
    }
    if pairs.iter().any(|(chosen, current)| chosen == current) {
        return Err(ChangeFailure::Same);
    }
    if pairs
        .iter()
        .any(|(chosen, current)| chosen.starts_with(current) || current.starts_with(chosen))
    {
        return Err(ChangeFailure::Nested);
    }
    check_private(chosen)?;
    let mut entries = std::fs::read_dir(chosen).map_err(|_| ChangeFailure::Inaccessible)?;
    if entries.next().is_none() {
        return Ok(FolderContents::Empty);
    }
    if JobStore::is_store(chosen) {
        return Ok(FolderContents::Store);
    }
    // 保存先でないのに、保存先の中のフォルダと同じ名前のフォルダ（ほかの保存先など）があれば、使わない（混ざるため）
    if JobStore::has_store_entries(chosen) {
        return Err(ChangeFailure::NameConflict);
    }
    Ok(FolderContents::Other)
}

/// フォルダ `folder` が、利用者だけが読み書きできるこの PC のフォルダかを確かめる
///
/// # Errors
///
/// ネットワークのフォルダ・権限を付けられないドライブ・ほかの利用者も読み書きできるフォルダ・確かめられないフォルダの
/// 場合。
pub(crate) fn check_private(folder: &Path) -> Result<(), ChangeFailure> {
    platform_foundation::folder::check_private_folder(folder).map_err(|error| match error {
        PrivateFolderError::NotOnThisPc => ChangeFailure::NotLocal,
        PrivateFolderError::NoPermissions => ChangeFailure::NoPermissions,
        PrivateFolderError::OpenToOthers => ChangeFailure::OpenToOthers,
        error @ PrivateFolderError::Io(_) => {
            eprintln!("{error}");
            ChangeFailure::Inaccessible
        }
    })
}

/// 起動のときに開いた保存先 `root` が、利用者だけが読み書きできるこの PC のフォルダかを確かめる
///
/// 既定の保存先 `default`（利用者ごとのデータのフォルダの下に、ジョブマネージャーが作るフォルダ）は、ほかの利用者も
/// 読み書きできれば、利用者だけが読み書きできるように直して使い続ける（前の版が作ったフォルダ・上のフォルダから
/// 権利を引き継いだフォルダのため）。利用者が選んだフォルダは直さない（ほかのデータと一緒に使っているおそれがあるため）。
///
/// # Errors
///
/// 満たさない（直せない）場合。
pub(crate) fn check_opened_store(root: &Path, default: Option<&Path>) -> Result<(), ChangeFailure> {
    match check_private(root) {
        Err(ChangeFailure::OpenToOthers)
            if default.is_some_and(|default| crate::same_folder(default, root)) =>
        {
            if let Err(error) = platform_foundation::folder::make_private(root) {
                eprintln!("cannot make the default job store private: {error}");
                return Err(ChangeFailure::OpenToOthers);
            }
            check_private(root)
        }
        result => result,
    }
}

/// 保存先 `store` に、待ち・計算中のジョブ（中断のジョブを待つものを除く）、または取り込みを待つ投入があるか
///
/// 読めない保存先は、あるとみなす（読めない間に替えると、待ちのジョブが残るため）。保存先のフォルダがない（外した
/// ドライブなど）ときは、ないとみなす。
pub(crate) fn is_busy(store: &JobStore) -> bool {
    let unreadable = |error: &StoreError| !matches!(error, StoreError::Io(error) if error.kind() == std::io::ErrorKind::NotFound);
    let jobs = match store.load_jobs() {
        Ok((jobs, _)) => {
            let idle = job_manager::idle_jobs(&jobs);
            jobs.iter().any(|job| !idle.contains(&job.job_id))
        }
        Err(error) => unreadable(&error),
    };
    let submissions = match store.inbox_submissions() {
        Ok(submissions) => submissions
            .iter()
            .any(|submission| submission.rejected.is_none()),
        Err(error) => unreadable(&error),
    };
    jobs || submissions
}

/// 保存先をフォルダ `chosen` に替え、ジョブマネージャーを起動し直す（このプロセスは、`requests` のサービスを終えて
/// 終わる。戻るのは替えられなかったときだけ）
///
/// # Errors
///
/// フォルダを保存先として開けない・設定を保存できない（今の保存先のまま）、起動し直せない（設定は
/// 書いた）場合。
pub(crate) fn switch(
    chosen: &Path,
    requests: &Sender<ServiceRequest>,
) -> Result<Infallible, ChangeFailure> {
    let path = chosen.to_str().ok_or(ChangeFailure::SettingNotSaved)?;
    let existed = JobStore::is_store(chosen);
    // 保存先として開けるかを確かめる（受付とジョブのフォルダを作る）
    JobStore::open(chosen).map_err(|error| {
        eprintln!("cannot open the new job store: {error}");
        ChangeFailure::Inaccessible
    })?;
    if let Err(error) = crate::settings::set(
        crate::STORE_KEY,
        serde_json::Value::String(path.to_string()),
    ) {
        eprintln!("cannot save the job store setting: {error}");
        // 開いたときに作った空のフォルダを消す（次に選んだときに、保存先とみなさないように）
        if !existed {
            JobStore::remove_empty_entries(chosen);
        }
        return Err(ChangeFailure::SettingNotSaved);
    }
    // Windows の起動のときのコマンドが保存先を指定していれば、設定の保存先で起動するように書き直す
    if let Some(command) = crate::autostart_command(None)
        && let Err(error) = crate::drop_autostart_store(&command)
    {
        eprintln!("cannot update the autostart command: {error}");
    }
    // 新しいジョブマネージャーは設定の保存先を開く（保存先が違うため、このプロセスのロック・トレイと重ならない）
    std::env::current_exe()
        .and_then(|exe| std::process::Command::new(exe).spawn())
        .map_err(|error| {
            eprintln!("cannot restart the job manager: {error}");
            ChangeFailure::RestartFailed
        })?;
    // 待ち・計算中のジョブがないため、サービスはすぐに終わる
    crate::exit_process(requests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_are_checked_against_the_current_store() {
        let root = std::env::temp_dir().join(format!(
            "redring-job-manager-store-change-test-{}",
            std::process::id()
        ));
        let current = root.join("current");
        JobStore::open(&current).unwrap();
        // 選ぶフォルダは、利用者だけが読み書きできるフォルダにする
        let private = |folder: &Path| {
            platform_foundation::folder::create_private_folder(folder).unwrap();
        };
        let empty = root.join("empty");
        private(&empty);
        let other = root.join("other");
        private(&other);
        std::fs::write(other.join("notes.txt"), b"notes").unwrap();
        let store = root.join("store");
        JobStore::open(&store).unwrap();

        assert_eq!(check_folder(&current, &empty), Ok(FolderContents::Empty));
        assert_eq!(check_folder(&current, &other), Ok(FolderContents::Other));
        assert_eq!(check_folder(&current, &store), Ok(FolderContents::Store));
        assert_eq!(check_folder(&current, &current), Err(ChangeFailure::Same));
        // 今の保存先の中・今の保存先を含むフォルダは使わない
        assert_eq!(
            check_folder(&current, &current.join("jobs")),
            Err(ChangeFailure::Nested)
        );
        assert_eq!(check_folder(&current, &root), Err(ChangeFailure::Nested));
        // ほかの保存先を含むフォルダ（中に保存先と同じ名前のフォルダがある）は使わない
        let parent = root.join("parent");
        private(&parent);
        JobStore::open(parent.join("jobs")).unwrap();
        assert_eq!(
            check_folder(&current, &parent),
            Err(ChangeFailure::NameConflict)
        );
        assert_eq!(
            check_folder(&current, &root.join("missing")),
            Err(ChangeFailure::Inaccessible)
        );
        // 待ちのジョブも投入もない保存先
        let current_store = JobStore::open(&current).unwrap();
        assert!(!is_busy(&current_store));
        // 取り込みを待つ投入がある保存先
        std::fs::create_dir_all(current.join("inbox").join("submission")).unwrap();
        assert!(is_busy(&current_store));
        // 開いた保存先は、利用者だけが読み書きできるフォルダ（作ったときから）
        assert_eq!(check_opened_store(&current, None), Ok(()));
        assert_eq!(check_opened_store(&current, Some(&current)), Ok(()));
        assert_eq!(
            check_opened_store(&root.join("missing"), None),
            Err(ChangeFailure::Inaccessible)
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}
