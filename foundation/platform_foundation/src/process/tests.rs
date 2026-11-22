use std::process::Command;

use super::{ChildGroup, hide_console_window};

/// すぐに正常に終わるコマンド
fn quick_command() -> Command {
    if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.args(["/C", "exit 0"]);
        command
    } else {
        Command::new("true")
    }
}

#[test]
fn a_child_in_the_group_runs_to_the_end() {
    let group = ChildGroup::new().unwrap();
    let mut command = quick_command();
    hide_console_window(&mut command);
    let (mut child, ungrouped) = group.spawn(&mut command).unwrap();
    assert!(child.wait().unwrap().success());
    assert!(ungrouped.is_none());
}

#[cfg(windows)]
#[test]
fn children_in_the_group_stop_when_the_group_is_dropped() {
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let group = ChildGroup::new().unwrap();
    // 30 秒ほどかかるコマンド
    let mut command = Command::new("ping");
    command
        .args(["-n", "30", "127.0.0.1"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let (mut child, ungrouped) = group.spawn(&mut command).unwrap();
    assert!(ungrouped.is_none());
    assert!(child.try_wait().unwrap().is_none());

    // 組を閉じて止めたときの終了の番号は 0 になるため、終わりまで動くより早く止まったことで確かめる
    drop(group);
    let deadline = Instant::now() + Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline, "the child did not stop");
        std::thread::sleep(Duration::from_millis(50));
    }
}
