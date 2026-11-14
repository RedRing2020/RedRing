use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use cam_sim::{Checkpoints, SavedRecord};

use super::{HEADER_LEN, ResumeFile, checksum};

/// 試験ごとのフォルダ（試験の終わりに消す）
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "redring-runner-resume-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        Self(path)
    }

    fn file(&self) -> PathBuf {
        self.0.join("resume").join("toolpath.resume")
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn record(index: u8) -> SavedRecord {
    SavedRecord {
        state: vec![index; 4],
        result: vec![index; usize::from(index) * 10],
    }
}

/// 開いて、前の実行の記録をすべて読み戻す
fn open_and_read(path: &std::path::Path, input: u64) -> (ResumeFile, Vec<SavedRecord>) {
    let mut file = ResumeFile::open(path, 1, input).unwrap();
    let saved = std::iter::from_fn(|| file.next_saved()).collect();
    (file, saved)
}

fn write_records(path: &std::path::Path, records: &[SavedRecord]) {
    let (mut file, saved) = open_and_read(path, 2);
    assert!(saved.is_empty());
    for record in records {
        file.record(&record.state, &record.result);
    }
}

#[test]
fn records_are_read_back_in_order_and_new_records_follow_them() {
    let dir = TempDir::new();
    write_records(&dir.file(), &[record(1), record(2)]);

    let (mut file, saved) = open_and_read(&dir.file(), 2);
    assert_eq!(saved, vec![record(1), record(2)]);
    assert_eq!(file.take_notes(), vec!["resume: continuing from 2 records"]);
    file.record(&record(3).state, &record(3).result);
    drop(file);

    let (_, saved) = open_and_read(&dir.file(), 2);
    assert_eq!(saved, vec![record(1), record(2), record(3)]);
}

#[test]
fn a_broken_record_and_the_records_after_it_are_discarded() {
    let dir = TempDir::new();
    write_records(&dir.file(), &[record(1), record(2), record(3)]);
    let mut bytes = fs::read(dir.file()).unwrap();
    // 2 つ目の記録の成果の 1 バイトを変える（検査の値が合わなくなる）
    let second_result = HEADER_LEN as usize + (16 + 4 + 10 + 8) + 16 + 4;
    bytes[second_result] ^= 0xff;
    fs::write(dir.file(), &bytes).unwrap();

    let (mut file, saved) = open_and_read(&dir.file(), 2);
    assert_eq!(saved, vec![record(1)]);
    assert!(file.take_notes()[0].starts_with("resume: discarded "));
    // 捨てた所から書き足す
    file.record(&record(4).state, &record(4).result);
    drop(file);
    let (_, saved) = open_and_read(&dir.file(), 2);
    assert_eq!(saved, vec![record(1), record(4)]);
}

#[test]
fn a_record_cut_off_while_writing_is_discarded() {
    let dir = TempDir::new();
    write_records(&dir.file(), &[record(1), record(2)]);
    let len = fs::metadata(dir.file()).unwrap().len();
    fs::OpenOptions::new()
        .write(true)
        .open(dir.file())
        .unwrap()
        .set_len(len - 3)
        .unwrap();

    let (_, saved) = open_and_read(&dir.file(), 2);
    assert_eq!(saved, vec![record(1)]);
}

#[test]
fn records_of_another_input_or_a_broken_header_are_discarded() {
    let dir = TempDir::new();
    write_records(&dir.file(), &[record(1)]);
    let (mut file, saved) = open_and_read(&dir.file(), 3);
    assert!(saved.is_empty());
    assert_eq!(
        file.take_notes(),
        vec!["resume: the input has changed; computing from the start"]
    );
    drop(file);

    fs::write(dir.file(), b"RRRES").unwrap();
    let (mut file, saved) = open_and_read(&dir.file(), 3);
    assert!(saved.is_empty());
    assert_eq!(
        file.take_notes(),
        vec!["resume: the header is broken; computing from the start"]
    );
}

#[test]
fn discarding_removes_every_record() {
    let dir = TempDir::new();
    write_records(&dir.file(), &[record(1), record(2)]);
    let mut file = ResumeFile::open(&dir.file(), 1, 2).unwrap();
    assert_eq!(file.next_saved(), Some(record(1)));
    file.discard("the records are broken");
    assert_eq!(file.next_saved(), None);
    file.record(&record(5).state, &record(5).result);
    drop(file);

    let (_, saved) = open_and_read(&dir.file(), 2);
    assert_eq!(saved, vec![record(5)]);
}

#[test]
fn records_not_read_back_are_dropped_when_a_new_record_is_written() {
    let dir = TempDir::new();
    write_records(&dir.file(), &[record(1), record(2)]);
    let mut file = ResumeFile::open(&dir.file(), 1, 2).unwrap();
    assert_eq!(file.next_saved(), Some(record(1)));
    file.record(&record(6).state, &record(6).result);
    drop(file);

    let (_, saved) = open_and_read(&dir.file(), 2);
    assert_eq!(saved, vec![record(1), record(6)]);
}

#[test]
fn the_checksum_is_fnv_1a() {
    assert_eq!(checksum(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(checksum(b"a"), 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn a_state_too_long_to_read_back_is_not_written() {
    let dir = TempDir::new();
    let mut file = ResumeFile::open(&dir.file(), 1, 2).unwrap();
    assert_eq!(file.next_saved(), None);
    file.record(&[0; 4097], b"result");
    // 書けなくなった後は書かない（読み戻せない記録を残さない）
    file.record(&record(1).state, &record(1).result);
    assert_eq!(
        file.take_notes(),
        vec!["resume: cannot write record 0: the state is too long; continuing without recording"]
    );
    drop(file);
    let (_, saved) = open_and_read(&dir.file(), 2);
    assert!(saved.is_empty());
}
