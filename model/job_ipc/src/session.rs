//! 要求を送る側（[`Client`]）と受ける側（[`serve`]。設計: JOB_IPC_DESIGN.md 4.2・5 章）
//!
//! 接続の最初に口の版を確かめ、その後は 1 つの要求に 1 つの応答を順に行う。

use std::fmt;
use std::io::{self, Read, Write};

use crate::frame::{
    MAX_FILE_BYTES, MAX_INPUT_BYTES, MAX_REQUEST_JSON_BYTES, MAX_RESPONSE_JSON_BYTES, read_data,
    read_json, write_data, write_json,
};
use crate::protocol::{
    ErrorKind, InputHeader, JobFile, JobSummary, PROTOCOL_VERSION, PendingSubmission, Request,
    Response, SubmitJob,
};

/// 1 つの投入の入力の数の上限
pub const MAX_INPUTS: usize = 8;

/// 受ける側の誤り（種類と、英語の詳しい理由）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerError {
    pub kind: ErrorKind,
    pub message: String,
}

impl HandlerError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// ジョブのファイルの中身（大きさと、読む口）
pub struct FileContent {
    pub size: u64,
    pub reader: Box<dyn Read + Send>,
}

/// 要求を処理する側
pub trait Handler {
    /// 入力 `inputs`（名前と中身）とともに、ジョブの列 `jobs` を投入し、投入の ID を返す
    fn submit(
        &mut self,
        jobs: Vec<SubmitJob>,
        inputs: Vec<(String, Vec<u8>)>,
    ) -> Result<String, HandlerError>;

    /// ジョブの一覧（番号の順）
    fn jobs(&mut self) -> Result<Vec<JobSummary>, HandlerError>;

    /// まだジョブになっていない投入
    fn pending_submissions(&mut self) -> Result<Vec<PendingSubmission>, HandlerError>;

    /// ジョブ `job_id` のファイル `file` の中身
    fn read_job_file(&mut self, job_id: u64, file: JobFile) -> Result<FileContent, HandlerError>;

    /// ジョブマネージャーのウィンドウを開く
    fn open_window(&mut self) -> Result<(), HandlerError>;

    /// 切削シミュレーションのジョブ `simulation_job_id` の結果（その CL）を、投入した側が取り込んだと記録する
    fn record_imported(&mut self, simulation_job_id: u64) -> Result<(), HandlerError>;

    /// ジョブ `job_ids` を取り消すよう頼む（取り消し終えるのを待たない）
    fn cancel(&mut self, job_ids: Vec<u64>) -> Result<(), HandlerError>;
}

fn error_response(kind: ErrorKind, message: impl Into<String>) -> Response {
    Response::Error {
        kind,
        message: message.into(),
    }
}

/// 接続 `connection` の要求を、`handler` で処理する（接続が閉じるまで。版が違う・正しくない要求のときは、誤りを
/// 返して終える）。`manager_version` は、版を確かめたときに返すジョブマネージャーの版
///
/// # Errors
///
/// 接続を読み書きできない・中身の大きさが合わない場合。
pub fn serve(
    connection: &mut (impl Read + Write),
    handler: &mut impl Handler,
    manager_version: &str,
) -> io::Result<()> {
    match read_json::<Request>(connection, MAX_REQUEST_JSON_BYTES) {
        Ok(Some(Request::Hello { protocol_version })) if protocol_version == PROTOCOL_VERSION => {
            write_json(
                connection,
                &Response::Hello {
                    protocol_version: PROTOCOL_VERSION,
                    manager_version: manager_version.to_string(),
                },
            )?;
        }
        Ok(Some(Request::Hello { protocol_version })) => {
            return write_json(
                connection,
                &error_response(
                    ErrorKind::UnsupportedVersion,
                    format!(
                        "protocol version {protocol_version} is not supported (expected {PROTOCOL_VERSION})"
                    ),
                ),
            );
        }
        Ok(Some(_)) => {
            return write_json(
                connection,
                &error_response(ErrorKind::InvalidRequest, "the first request must be hello"),
            );
        }
        Ok(None) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            return write_json(
                connection,
                &error_response(ErrorKind::InvalidRequest, error.to_string()),
            );
        }
        Err(error) => return Err(error),
    }
    loop {
        let request = match read_json::<Request>(connection, MAX_REQUEST_JSON_BYTES) {
            Ok(Some(request)) => request,
            Ok(None) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                return write_json(
                    connection,
                    &error_response(ErrorKind::InvalidRequest, error.to_string()),
                );
            }
            Err(error) => return Err(error),
        };
        let reply = |result: Result<Response, HandlerError>| {
            result.unwrap_or_else(|error| error_response(error.kind, error.message))
        };
        let response = match request {
            Request::Hello { .. } => {
                error_response(ErrorKind::InvalidRequest, "hello is sent only once")
            }
            Request::Submit { jobs, inputs } => {
                // 上限を超える入力は読まずに、接続を終える（続く中身を読み捨てない）
                let total = inputs
                    .iter()
                    .try_fold(0u64, |total, input| total.checked_add(input.size));
                if inputs.len() > MAX_INPUTS || total.is_none_or(|total| total > MAX_INPUT_BYTES) {
                    return write_json(
                        connection,
                        &error_response(ErrorKind::TooLarge, "the submission inputs are too large"),
                    );
                }
                let mut received = Vec::with_capacity(inputs.len());
                for input in inputs {
                    let bytes = read_data(connection, input.size, MAX_INPUT_BYTES)?;
                    received.push((input.name, bytes));
                }
                reply(
                    handler
                        .submit(jobs, received)
                        .map(|submission_id| Response::Submitted { submission_id }),
                )
            }
            Request::Jobs => reply(handler.jobs().map(|jobs| Response::Jobs { jobs })),
            Request::PendingSubmissions => reply(
                handler
                    .pending_submissions()
                    .map(|submissions| Response::PendingSubmissions { submissions }),
            ),
            Request::ReadJobFile { job_id, file } => match handler.read_job_file(job_id, file) {
                Ok(content) if content.size > MAX_FILE_BYTES => {
                    error_response(ErrorKind::TooLarge, "the job file is too large")
                }
                Ok(mut content) => {
                    write_json(connection, &Response::JobFile { size: content.size })?;
                    write_data(connection, &mut content.reader, content.size)?;
                    connection.flush()?;
                    continue;
                }
                Err(error) => error_response(error.kind, error.message),
            },
            Request::OpenWindow => reply(handler.open_window().map(|()| Response::Ok)),
            Request::RecordImported { simulation_job_id } => reply(
                handler
                    .record_imported(simulation_job_id)
                    .map(|()| Response::Ok),
            ),
            Request::Cancel { job_ids } => reply(handler.cancel(job_ids).map(|()| Response::Ok)),
        };
        write_json(connection, &response)?;
        connection.flush()?;
    }
}

/// 要求を送る側の誤り
#[derive(Debug)]
pub enum ClientError {
    /// 接続を読み書きできない・送る中身が上限を超える
    Io(io::Error),
    /// ジョブマネージャーが返した誤り
    Remote { kind: ErrorKind, message: String },
    /// 要求に合わない応答
    UnexpectedResponse,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Remote { kind, message } => write!(f, "{kind:?}: {message}"),
            Self::UnexpectedResponse => write!(f, "unexpected response from the job manager"),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<io::Error> for ClientError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// 要求を送る側（版を確かめた接続）
#[derive(Debug)]
pub struct Client<C> {
    connection: C,
    manager_version: String,
}

impl<C: Read + Write> Client<C> {
    /// 接続 `connection` で口の版を確かめる
    ///
    /// # Errors
    ///
    /// 読み書きできない・版が違う（`ErrorKind::UnsupportedVersion`）場合。
    pub fn new(mut connection: C) -> Result<Self, ClientError> {
        write_json(
            &mut connection,
            &Request::Hello {
                protocol_version: PROTOCOL_VERSION,
            },
        )?;
        connection.flush()?;
        match read_response(&mut connection)? {
            Response::Hello {
                protocol_version,
                manager_version,
            } if protocol_version == PROTOCOL_VERSION => Ok(Self {
                connection,
                manager_version,
            }),
            Response::Hello {
                protocol_version, ..
            } => Err(ClientError::Remote {
                kind: ErrorKind::UnsupportedVersion,
                message: format!("the job manager speaks protocol version {protocol_version}"),
            }),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// 版を確かめたときに返った、ジョブマネージャーの版
    pub fn manager_version(&self) -> &str {
        &self.manager_version
    }

    fn call(&mut self, request: &Request) -> Result<Response, ClientError> {
        write_json(&mut self.connection, request)?;
        self.connection.flush()?;
        read_response(&mut self.connection)
    }

    /// 入力 `inputs`（名前と中身）とともに、ジョブの列 `jobs` を投入し、投入の ID を返す
    ///
    /// # Errors
    ///
    /// 入力が上限を超える・読み書きできない・ジョブマネージャーが誤りを返した場合。
    pub fn submit(
        &mut self,
        jobs: Vec<SubmitJob>,
        inputs: &[(&str, &[u8])],
    ) -> Result<String, ClientError> {
        let total: u64 = inputs.iter().map(|(_, bytes)| bytes.len() as u64).sum();
        if inputs.len() > MAX_INPUTS || total > MAX_INPUT_BYTES {
            return Err(ClientError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the submission inputs are too large",
            )));
        }
        let headers = inputs
            .iter()
            .map(|(name, bytes)| InputHeader {
                name: (*name).to_string(),
                size: bytes.len() as u64,
            })
            .collect();
        write_json(
            &mut self.connection,
            &Request::Submit {
                jobs,
                inputs: headers,
            },
        )?;
        for (_, bytes) in inputs {
            write_data(&mut self.connection, &mut &bytes[..], bytes.len() as u64)?;
        }
        self.connection.flush()?;
        match read_response(&mut self.connection)? {
            Response::Submitted { submission_id } => Ok(submission_id),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// ジョブの一覧（番号の順）
    ///
    /// # Errors
    ///
    /// 読み書きできない・ジョブマネージャーが誤りを返した場合。
    pub fn jobs(&mut self) -> Result<Vec<JobSummary>, ClientError> {
        match self.call(&Request::Jobs)? {
            Response::Jobs { jobs } => Ok(jobs),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// まだジョブになっていない投入
    ///
    /// # Errors
    ///
    /// 読み書きできない・ジョブマネージャーが誤りを返した場合。
    pub fn pending_submissions(&mut self) -> Result<Vec<PendingSubmission>, ClientError> {
        match self.call(&Request::PendingSubmissions)? {
            Response::PendingSubmissions { submissions } => Ok(submissions),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// ジョブ `job_id` のファイル `file` の中身
    ///
    /// # Errors
    ///
    /// 読み書きできない・大きさが合わない・ジョブマネージャーが誤り（ないジョブ・ないファイルなど）を返した場合。
    pub fn read_job_file(&mut self, job_id: u64, file: JobFile) -> Result<Vec<u8>, ClientError> {
        match self.call(&Request::ReadJobFile { job_id, file })? {
            Response::JobFile { size } => {
                Ok(read_data(&mut self.connection, size, MAX_FILE_BYTES)?)
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// ジョブマネージャーのウィンドウを開く
    ///
    /// # Errors
    ///
    /// 読み書きできない・ジョブマネージャーが誤りを返した場合。
    pub fn open_window(&mut self) -> Result<(), ClientError> {
        match self.call(&Request::OpenWindow)? {
            Response::Ok => Ok(()),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// 切削シミュレーションのジョブ `simulation_job_id` の結果（その CL）を取り込んだことを伝える
    ///
    /// # Errors
    ///
    /// 読み書きできない・ジョブマネージャーが誤り（ないジョブなど）を返した場合。
    pub fn record_imported(&mut self, simulation_job_id: u64) -> Result<(), ClientError> {
        match self.call(&Request::RecordImported { simulation_job_id })? {
            Response::Ok => Ok(()),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// ジョブ `job_ids` を取り消すよう頼む（計算待ちは取り消し、計算中はランナーを止める。取り消し終えるのを
    /// 待たない）
    ///
    /// # Errors
    ///
    /// 読み書きできない・ジョブマネージャーが誤り（取り消しを受けられない）を返した場合。
    pub fn cancel(&mut self, job_ids: Vec<u64>) -> Result<(), ClientError> {
        match self.call(&Request::Cancel { job_ids })? {
            Response::Ok => Ok(()),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }
}

/// 応答を読む（誤りの応答は `ClientError::Remote`、接続が閉じていれば `UnexpectedEof`）
fn read_response(connection: &mut impl Read) -> Result<Response, ClientError> {
    match read_json::<Response>(connection, MAX_RESPONSE_JSON_BYTES)? {
        Some(Response::Error { kind, message }) => Err(ClientError::Remote { kind, message }),
        Some(response) => Ok(response),
        None => Err(ClientError::Io(io::ErrorKind::UnexpectedEof.into())),
    }
}
