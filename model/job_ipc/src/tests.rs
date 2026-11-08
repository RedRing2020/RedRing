use std::io::{self, Cursor, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::frame::{
    FrameKind, MAX_CHUNK_BYTES, MAX_REQUEST_JSON_BYTES, MAX_RESPONSE_JSON_BYTES, read_data,
    read_frame, read_json, write_data, write_frame, write_json,
};
use crate::*;

/// 試験ごとの口の名前（利用者の実際の口と分ける）
fn test_endpoint() -> Endpoint {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    Endpoint::with_name(&format!(
        "test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
    .unwrap()
}

/// メモリの上の接続（読むものを決めておき、書いたものを残す）
struct Duplex {
    input: Cursor<Vec<u8>>,
    output: Vec<u8>,
}

impl Read for Duplex {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.input.read(buffer)
    }
}

impl Write for Duplex {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.output.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// 受け手が受けた投入（ジョブの列と、入力の名前と中身）
type Submitted = Arc<Mutex<Vec<(Vec<SubmitJob>, Vec<(String, Vec<u8>)>)>>>;

/// 投入を覚え、決まったジョブとファイルを返す受け手
#[derive(Default)]
struct FakeHandler {
    submitted: Submitted,
    opened: Arc<AtomicUsize>,
}

/// 受け手が返す成果物（小分けを 3 つ以上にする大きさ）
fn result_bytes() -> Vec<u8> {
    (0..MAX_CHUNK_BYTES * 2 + 7)
        .map(|i| (i % 251) as u8)
        .collect()
}

impl Handler for FakeHandler {
    fn submit(
        &mut self,
        jobs: Vec<SubmitJob>,
        inputs: Vec<(String, Vec<u8>)>,
    ) -> Result<String, HandlerError> {
        self.submitted.lock().unwrap().push((jobs, inputs));
        Ok("submission-1".to_string())
    }

    fn jobs(&mut self) -> Result<Vec<JobSummary>, HandlerError> {
        Ok(vec![JobSummary {
            job_id: 1,
            kind: JobKind::Cam,
            status: JobStatus::Succeeded,
            depends_on: Vec::new(),
            submission_id: "submission-1".to_string(),
            source: None,
            has_input_toolpath: false,
            has_result: true,
        }])
    }

    fn pending_submissions(&mut self) -> Result<Vec<PendingSubmission>, HandlerError> {
        Ok(vec![PendingSubmission {
            source: None,
            rejected: true,
        }])
    }

    fn read_job_file(&mut self, job_id: u64, file: JobFile) -> Result<FileContent, HandlerError> {
        match (job_id, file) {
            (1, JobFile::Result) => {
                let bytes = result_bytes();
                Ok(FileContent {
                    size: bytes.len() as u64,
                    reader: Box::new(Cursor::new(bytes)),
                })
            }
            (1, _) => Err(HandlerError::new(ErrorKind::FileNotFound, "no such file")),
            _ => Err(HandlerError::new(ErrorKind::JobNotFound, "no such job")),
        }
    }

    fn open_window(&mut self) -> Result<(), HandlerError> {
        self.opened.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[test]
fn frames_are_written_and_read_back_within_their_limits() {
    let mut bytes = Vec::new();
    write_json(&mut bytes, &Request::Jobs).unwrap();
    write_data(&mut bytes, &mut &[1u8, 2, 3][..], 3).unwrap();
    let mut reader = Cursor::new(bytes);
    assert_eq!(
        read_json::<Request>(&mut reader, MAX_RESPONSE_JSON_BYTES).unwrap(),
        Some(Request::Jobs)
    );
    assert_eq!(read_data(&mut reader, 3, 3).unwrap(), vec![1, 2, 3]);
    assert!(
        read_frame(&mut reader, MAX_REQUEST_JSON_BYTES)
            .unwrap()
            .is_none()
    );

    // 上限を超える JSON・知らない種類・途中で終わった枠・大きさの合わない中身は、誤り
    assert!(
        write_frame(
            &mut Vec::new(),
            FrameKind::Json,
            &vec![b' '; MAX_RESPONSE_JSON_BYTES + 1]
        )
        .is_err()
    );
    // 要求の JSON は、応答より小さい上限で読む
    let mut oversized = vec![0u8];
    oversized.extend_from_slice(&(MAX_REQUEST_JSON_BYTES as u32 + 1).to_le_bytes());
    assert!(read_frame(&mut Cursor::new(oversized.clone()), MAX_REQUEST_JSON_BYTES).is_err());
    assert!(matches!(
        read_frame(&mut Cursor::new(oversized), MAX_RESPONSE_JSON_BYTES),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof
    ));
    assert!(
        read_frame(
            &mut Cursor::new(vec![9u8, 0, 0, 0, 0]),
            MAX_REQUEST_JSON_BYTES
        )
        .is_err()
    );
    assert!(
        read_frame(
            &mut Cursor::new(vec![0u8, 4, 0, 0, 0, b'{']),
            MAX_REQUEST_JSON_BYTES
        )
        .is_err()
    );
    let mut short = Vec::new();
    write_data(&mut short, &mut &[1u8, 2][..], 2).unwrap();
    assert!(read_data(&mut Cursor::new(short.clone()), 3, 3).is_err());
    assert!(read_data(&mut Cursor::new(short.clone()), 1, 3).is_err());
    assert!(read_data(&mut Cursor::new(short), 2, 1).is_err());
}

#[test]
fn a_different_protocol_version_is_refused() {
    let mut input = Vec::new();
    write_json(
        &mut input,
        &Request::Hello {
            protocol_version: PROTOCOL_VERSION + 1,
        },
    )
    .unwrap();
    write_json(&mut input, &Request::Jobs).unwrap();
    let mut connection = Duplex {
        input: Cursor::new(input),
        output: Vec::new(),
    };
    serve(&mut connection, &mut FakeHandler::default(), "0.0.0").unwrap();
    let mut output = Cursor::new(connection.output);
    assert!(matches!(
        read_json::<Response>(&mut output, MAX_RESPONSE_JSON_BYTES).unwrap(),
        Some(Response::Error {
            kind: ErrorKind::UnsupportedVersion,
            ..
        })
    ));
    // 版が違えば、続く要求に答えない
    assert!(
        read_frame(&mut output, MAX_REQUEST_JSON_BYTES)
            .unwrap()
            .is_none()
    );
}

#[test]
fn requests_must_start_with_hello() {
    let mut input = Vec::new();
    write_json(&mut input, &Request::Jobs).unwrap();
    let mut connection = Duplex {
        input: Cursor::new(input),
        output: Vec::new(),
    };
    serve(&mut connection, &mut FakeHandler::default(), "0.0.0").unwrap();
    assert!(matches!(
        read_json::<Response>(&mut Cursor::new(connection.output), MAX_RESPONSE_JSON_BYTES)
            .unwrap(),
        Some(Response::Error {
            kind: ErrorKind::InvalidRequest,
            ..
        })
    ));
}

#[test]
fn requests_go_through_the_endpoint() {
    let endpoint = test_endpoint();
    let mut listener = endpoint.listen().unwrap();
    let handler = FakeHandler::default();
    let (submitted, opened) = (Arc::clone(&handler.submitted), Arc::clone(&handler.opened));
    let server = std::thread::spawn(move || {
        let mut handler = handler;
        let mut connection = listener.accept().unwrap();
        serve(&mut connection, &mut handler, "1.2.3").unwrap();
    });

    let mut client = Client::new(endpoint.connect(Duration::from_secs(5)).unwrap()).unwrap();
    assert_eq!(client.manager_version(), "1.2.3");
    let job = SubmitJob {
        kind: JobKind::Sim,
        input_file: "sim.bin".to_string(),
        depends_on: vec![Dependency::Existing(1)],
        priority: 0,
        source: Some(JobSource {
            source_id: "op-1".to_string(),
            revision: 2,
            revision_key: Some("key".to_string()),
        }),
        toolpath_file: None,
    };
    let input = vec![7u8; MAX_CHUNK_BYTES + 1];
    assert_eq!(
        client
            .submit(vec![job.clone()], &[("sim.bin", &input)])
            .unwrap(),
        "submission-1"
    );
    assert_eq!(client.jobs().unwrap()[0].job_id, 1);
    assert!(client.pending_submissions().unwrap()[0].rejected);
    assert_eq!(
        client.read_job_file(1, JobFile::Result).unwrap(),
        result_bytes()
    );
    assert!(matches!(
        client.read_job_file(9, JobFile::Result),
        Err(ClientError::Remote {
            kind: ErrorKind::JobNotFound,
            ..
        })
    ));
    // 誤りの後も、同じ接続で続けられる
    client.open_window().unwrap();
    drop(client);
    server.join().unwrap();

    let submitted = submitted.lock().unwrap();
    assert_eq!(submitted.len(), 1);
    assert_eq!(submitted[0].0, vec![job]);
    assert_eq!(submitted[0].1, vec![("sim.bin".to_string(), input)]);
    assert_eq!(opened.load(Ordering::Relaxed), 1);
}

#[test]
fn connecting_to_a_closed_endpoint_fails_without_waiting() {
    let endpoint = test_endpoint();
    assert_eq!(
        endpoint.connect(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    // 閉じた口は、開き直せる
    drop(endpoint.listen().unwrap());
    let _listener = endpoint.listen().unwrap();
    // 開いている口は、2 つ目を開かない（UNIX では、開いているかを確かめるために一度つなぐため、接続を受ける試験と
    // 分ける）
    assert_eq!(
        endpoint.listen().unwrap_err().kind(),
        io::ErrorKind::AddrInUse
    );
    assert!(Endpoint::with_name("bad/name").is_err());
}
