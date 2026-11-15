//! ツールパス生成の途中の結果のファイル（計算の区切りごとの記録。設計: JOB_MANAGER_STORAGE_DESIGN.md 3.3）
//!
//! - 頭: 印 `RRRESUME`・形式の版（u32）・ランナーの実行ファイルの検査の値（u64）・入力のファイルの検査の値（u64）
//! - 続けて、区切りごとの記録: 通し番号（u64。0 から）・状態の長さ（u32）・成果の長さ（u32）・状態・成果・検査の値
//!   （u64。通し番号から成果まで）
//!
//! 数は little endian。検査の値は FNV-1a（64 ビット。書きかけ・偶然の壊れを見分けるためのもの）。状態と成果の中身は
//! 見ずに書き足し、読み戻す（中身は計算が確かめる）。

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use cam_sim::{Checkpoints, SavedRecord};

/// 頭の印
const MAGIC: &[u8; 8] = b"RRRESUME";
/// 形式の版
const FORMAT_VERSION: u32 = 1;
/// 頭のバイト数
const HEADER_LEN: u64 = 8 + 4 + 8 + 8;
/// 記録の、状態の前までのバイト数（通し番号・状態の長さ・成果の長さ）
const RECORD_HEAD_LEN: u64 = 8 + 4 + 4;
/// 検査の値のバイト数
const CHECKSUM_LEN: u64 = 8;
/// 状態の長さの上限（状態は、続けるための少しの数のため）
const MAX_STATE_LEN: usize = 4096;
/// ドライブへの書き込みを確かめる間隔（区切りごとに確かめると、短い区切りの多い計算で遅くなるため）
const SYNC_INTERVAL: Duration = Duration::from_secs(3);

/// 途中の結果のファイル
pub(crate) struct ResumeFile {
    file: File,
    runner_checksum: u64,
    input_checksum: u64,
    /// ファイルに頭がある（記録のないうちは頭も書かず、空のファイルにする）
    has_header: bool,
    /// 前の実行の記録を読み戻している間の、読み戻した所（読み戻し終えたら `None`）
    reading: Option<Reading>,
    /// 次に読み戻す・書く記録の通し番号
    next_sequence: u64,
    /// 前の実行から読み戻して使った記録の数を、ログに書いたか
    resumed_noted: bool,
    last_sync: Instant,
    /// 書けなくなった（その後は書かない）
    write_failed: bool,
    /// ランナーのログに書く行
    notes: Vec<String>,
}

impl ResumeFile {
    /// 途中の結果のファイル `path` を開く（なければ空のファイルを作る）
    ///
    /// 頭は、最初の記録を書くときに書く。記録のない途中の結果は、空のファイルにする（中を読まなくても、記録があるか
    /// が分かるように）。
    ///
    /// 頭のランナーの実行ファイルの検査の値 `runner_checksum`・入力のファイルの検査の値 `input_checksum` が今の計算と
    /// 違えば、前の記録を捨てる。記録は、読み戻すときに、通し番号が 0 から続き、検査の値が合うところまでを使い、その
    /// 後ろは捨てる。
    pub(crate) fn open(path: &Path, runner_checksum: u64, input_checksum: u64) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let mut notes = Vec::new();
        let len = file.metadata()?.len();
        let has_header = if len == 0 {
            false
        } else if let Some(reason) = check_header(&mut file, runner_checksum, input_checksum)? {
            notes.push(format!("resume: {reason}; computing from the start"));
            file.set_len(0)?;
            false
        } else {
            true
        };
        Ok(Self {
            file,
            runner_checksum,
            input_checksum,
            has_header,
            reading: has_header.then_some(Reading {
                len,
                end: HEADER_LEN,
            }),
            next_sequence: 0,
            resumed_noted: false,
            last_sync: Instant::now(),
            write_failed: false,
            notes,
        })
    }

    /// ランナーのログに書く行（取り出す）。前の実行から読み戻して使った記録があれば、その数も書く
    pub(crate) fn take_notes(&mut self) -> Vec<String> {
        self.note_resumed();
        std::mem::take(&mut self.notes)
    }

    /// 前の実行から読み戻して使った記録の数を、まだ書いていなければ書く（読み戻し終えた後だけ。計算が記録を捨てたら
    /// 数は 0 のため書かない）
    fn note_resumed(&mut self) {
        if self.reading.is_none() && !self.resumed_noted && self.next_sequence > 0 {
            self.resumed_noted = true;
            self.notes.push(format!(
                "resume: continuing from {} records",
                self.next_sequence
            ));
        }
    }

    /// 前の実行の記録の読み戻しを終える。読み戻した所より後ろ（読み戻していない記録・壊れた記録）を捨てる。使える
    /// 記録がなければ、空にする
    fn finish_reading(&mut self) -> io::Result<()> {
        if let Some(Reading { len, end }) = self.reading.take() {
            if self.next_sequence == 0 {
                self.empty()?;
            } else if end < len {
                self.file.set_len(end)?;
            }
        }
        Ok(())
    }

    /// 記録をすべて捨て、空のファイルにする
    fn empty(&mut self) -> io::Result<()> {
        self.has_header = false;
        self.file.set_len(0)
    }

    /// 読み戻した所 `reading` の次の記録を読み、記録とその長さを返す。通し番号が続き検査の値の合う記録がなければ
    /// `None`
    fn read_record(&mut self, reading: Reading) -> io::Result<Option<(SavedRecord, u64)>> {
        let left = reading.len - reading.end;
        self.file.seek(SeekFrom::Start(reading.end))?;
        if left < RECORD_HEAD_LEN + CHECKSUM_LEN {
            return Ok(None);
        }
        let mut head = [0; RECORD_HEAD_LEN as usize];
        self.file.read_exact(&mut head)?;
        let sequence = u64::from_le_bytes(head[..8].try_into().expect("8 bytes"));
        let state_len = u32::from_le_bytes(head[8..12].try_into().expect("4 bytes")) as usize;
        let result_len = u32::from_le_bytes(head[12..16].try_into().expect("4 bytes")) as usize;
        // 長さは、ファイルの残りで確かめてから確保する
        let record_len = RECORD_HEAD_LEN + (state_len + result_len) as u64 + CHECKSUM_LEN;
        if sequence != self.next_sequence || state_len > MAX_STATE_LEN || record_len > left {
            return Ok(None);
        }
        let mut state = vec![0; state_len];
        let mut result = vec![0; result_len];
        let mut checksum = [0; CHECKSUM_LEN as usize];
        self.file.read_exact(&mut state)?;
        self.file.read_exact(&mut result)?;
        self.file.read_exact(&mut checksum)?;
        let expected = fnv1a(fnv1a(fnv1a(FNV_OFFSET, &head), &state), &result);
        if u64::from_le_bytes(checksum) != expected {
            return Ok(None);
        }
        Ok(Some((SavedRecord { state, result }, record_len)))
    }

    fn append(&mut self, state: &[u8], result: &[u8]) -> io::Result<()> {
        if state.len() > MAX_STATE_LEN {
            return Err(io::Error::other("the state is too long"));
        }
        self.finish_reading()?;
        self.note_resumed();
        if !self.has_header {
            write_header(&mut self.file, self.runner_checksum, self.input_checksum)?;
            self.has_header = true;
        }
        let mut bytes = Vec::with_capacity(
            RECORD_HEAD_LEN as usize + state.len() + result.len() + CHECKSUM_LEN as usize,
        );
        bytes.extend_from_slice(&self.next_sequence.to_le_bytes());
        bytes.extend_from_slice(&length(state)?.to_le_bytes());
        bytes.extend_from_slice(&length(result)?.to_le_bytes());
        bytes.extend_from_slice(state);
        bytes.extend_from_slice(result);
        let checksum = fnv1a(FNV_OFFSET, &bytes);
        bytes.extend_from_slice(&checksum.to_le_bytes());
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&bytes)?;
        self.next_sequence += 1;
        if self.last_sync.elapsed() >= SYNC_INTERVAL {
            self.file.sync_data()?;
            self.last_sync = Instant::now();
        }
        Ok(())
    }
}

impl Checkpoints for ResumeFile {
    fn next_saved(&mut self) -> Option<SavedRecord> {
        let reading = self.reading?;
        match self.read_record(reading) {
            Ok(Some((record, record_len))) => {
                self.reading = Some(Reading {
                    end: reading.end + record_len,
                    ..reading
                });
                self.next_sequence += 1;
                return Some(record);
            }
            Ok(None) => {}
            Err(error) => self.notes.push(format!(
                "resume: cannot read record {}: {error}",
                self.next_sequence
            )),
        }
        // 読めた記録までを使い、その後ろは捨てる
        if reading.end < reading.len {
            self.notes.push(format!(
                "resume: discarded {} broken bytes after {} records",
                reading.len - reading.end,
                self.next_sequence
            ));
        }
        if let Err(error) = self.finish_reading() {
            self.write_failed = true;
            self.notes.push(format!(
                "resume: cannot discard the broken records: {error}"
            ));
        }
        None
    }

    fn discard(&mut self, reason: &str) {
        self.notes
            .push(format!("resume: {reason}; computing from the start"));
        self.reading = None;
        self.next_sequence = 0;
        if let Err(error) = self.empty() {
            self.write_failed = true;
            self.notes
                .push(format!("resume: cannot discard the records: {error}"));
        }
    }

    fn record(&mut self, state: &[u8], result: &[u8]) {
        if self.write_failed {
            return;
        }
        if let Err(error) = self.append(state, result) {
            // 途中の結果は補助のため、書けなくても計算は続ける（書きかけの記録は、次に開くときに捨てる）
            self.write_failed = true;
            self.notes.push(format!(
                "resume: cannot write record {}: {error}; continuing without recording",
                self.next_sequence
            ));
        }
    }
}

/// 前の実行の記録を読み戻した所
#[derive(Debug, Clone, Copy)]
struct Reading {
    /// 開いたときのファイルの長さ
    len: u64,
    /// 読み戻した記録の終わりの位置
    end: u64,
}

fn write_header(file: &mut File, runner_checksum: u64, input_checksum: u64) -> io::Result<()> {
    let mut header = Vec::with_capacity(HEADER_LEN as usize);
    header.extend_from_slice(MAGIC);
    header.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    header.extend_from_slice(&runner_checksum.to_le_bytes());
    header.extend_from_slice(&input_checksum.to_le_bytes());
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&header)
}

/// 頭が今の計算と合わなければ、その理由（英語）
fn check_header(
    file: &mut File,
    runner_checksum: u64,
    input_checksum: u64,
) -> io::Result<Option<&'static str>> {
    let mut header = [0; HEADER_LEN as usize];
    file.seek(SeekFrom::Start(0))?;
    match file.read_exact(&mut header) {
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            return Ok(Some("the header is broken"));
        }
        other => other?,
    }
    let number = |range: std::ops::Range<usize>| {
        let mut bytes = [0; 8];
        bytes[..range.len()].copy_from_slice(&header[range]);
        u64::from_le_bytes(bytes)
    };
    Ok(if &header[..8] != MAGIC {
        Some("the file is not a resume file")
    } else if number(8..12) != u64::from(FORMAT_VERSION) {
        Some("the file has another format version")
    } else if number(12..20) != runner_checksum {
        Some("the runner has changed")
    } else if number(20..28) != input_checksum {
        Some("the input has changed")
    } else {
        None
    })
}

fn length(bytes: &[u8]) -> io::Result<u32> {
    u32::try_from(bytes.len()).map_err(|_| io::Error::other("a record is too long"))
}

/// FNV-1a の始めの値
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a（64 ビット）。`hash` に `bytes` を続けて足す
fn fnv1a(hash: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(hash, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

/// バイト列 `bytes` の検査の値
pub(crate) fn checksum(bytes: &[u8]) -> u64 {
    fnv1a(FNV_OFFSET, bytes)
}

/// ファイル `path` の検査の値（読みながら足す）
pub(crate) fn file_checksum(path: &Path) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0; 1 << 16];
    let mut hash = FNV_OFFSET;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            return Ok(hash);
        }
        hash = fnv1a(hash, &buffer[..read]);
    }
}

#[cfg(test)]
mod tests;
