use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefScheme {
    Input,
    Result,
    Log,
}

impl RefScheme {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Input => "input://",
            Self::Result => "result://",
            Self::Log => "log://",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Result => "result",
            Self::Log => "log",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefValidationError {
    EmptyValue,
    InvalidScheme { expected: RefScheme, actual: String },
    NonAscii { value: String },
    EmptySegment { value: String },
    InvalidSegment { value: String },
    MissingDomain { value: String },
    DomainMismatch { expected: String, actual: String },
    MissingJobId { value: String },
    InvalidJobId { value: String },
    MissingSuffix { value: String },
}

impl Display for RefValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyValue => write!(f, "reference value is required"),
            Self::InvalidScheme { expected, actual } => write!(
                f,
                "invalid reference scheme: expected={}://, actual={}",
                expected.as_str(),
                actual
            ),
            Self::NonAscii { value } => write!(f, "reference must be ASCII: {}", value),
            Self::EmptySegment { value } => {
                write!(f, "reference contains empty path segment: {}", value)
            }
            Self::InvalidSegment { value } => {
                write!(f, "reference segment is invalid: {}", value)
            }
            Self::MissingDomain { value } => write!(f, "reference domain is missing: {}", value),
            Self::DomainMismatch { expected, actual } => {
                write!(
                    f,
                    "reference domain mismatch: expected={}, actual={}",
                    expected, actual
                )
            }
            Self::MissingJobId { value } => write!(f, "reference job id is missing: {}", value),
            Self::InvalidJobId { value } => write!(f, "reference job id is invalid: {}", value),
            Self::MissingSuffix { value } => write!(f, "reference suffix is missing: {}", value),
        }
    }
}

impl Error for RefValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRef {
    raw: String,
    scheme: RefScheme,
    segments: Vec<String>,
}

impl ParsedRef {
    pub fn as_str(&self) -> &str {
        self.raw.as_str()
    }

    pub fn scheme(&self) -> RefScheme {
        self.scheme
    }

    pub fn segments(&self) -> &[String] {
        self.segments.as_slice()
    }
}

pub struct RefFactory;

impl RefFactory {
    pub fn input(domain: &str, suffix_segments: &[&str]) -> Result<String, RefValidationError> {
        let mut segments = Vec::with_capacity(1 + suffix_segments.len());
        segments.push(domain);
        segments.extend_from_slice(suffix_segments);
        build_ref(RefScheme::Input, &segments)
    }

    pub fn result(
        domain: &str,
        job_id: u64,
        suffix_segments: &[&str],
    ) -> Result<String, RefValidationError> {
        let job_id_str = job_id.to_string();
        let mut segments = Vec::with_capacity(2 + suffix_segments.len());
        segments.push(domain);
        segments.push(job_id_str.as_str());
        segments.extend_from_slice(suffix_segments);
        build_ref(RefScheme::Result, &segments)
    }

    pub fn log(
        domain: &str,
        job_id: u64,
        suffix_segments: &[&str],
    ) -> Result<String, RefValidationError> {
        let job_id_str = job_id.to_string();
        let mut segments = Vec::with_capacity(2 + suffix_segments.len());
        segments.push(domain);
        segments.push(job_id_str.as_str());
        segments.extend_from_slice(suffix_segments);
        build_ref(RefScheme::Log, &segments)
    }
}

pub struct RefParser;

impl RefParser {
    pub fn parse_input(value: &str) -> Result<ParsedRef, RefValidationError> {
        parse_ref(value, RefScheme::Input)
    }

    pub fn parse_result(value: &str) -> Result<ParsedRef, RefValidationError> {
        parse_ref(value, RefScheme::Result)
    }

    pub fn parse_log(value: &str) -> Result<ParsedRef, RefValidationError> {
        parse_ref(value, RefScheme::Log)
    }

    pub fn parse_result_job_ref(
        value: &str,
        expected_domain: &str,
    ) -> Result<(u64, String), RefValidationError> {
        let parsed = Self::parse_result(value)?;
        let segments = parsed.segments();

        let domain = segments
            .first()
            .ok_or_else(|| RefValidationError::MissingDomain {
                value: parsed.as_str().to_string(),
            })?;
        if domain != expected_domain {
            return Err(RefValidationError::DomainMismatch {
                expected: expected_domain.to_string(),
                actual: domain.clone(),
            });
        }

        let job_id_text = segments
            .get(1)
            .ok_or_else(|| RefValidationError::MissingJobId {
                value: parsed.as_str().to_string(),
            })?;
        let job_id = job_id_text
            .parse::<u64>()
            .map_err(|_| RefValidationError::InvalidJobId {
                value: parsed.as_str().to_string(),
            })?;

        let suffix_segments =
            segments
                .get(2..)
                .ok_or_else(|| RefValidationError::MissingSuffix {
                    value: parsed.as_str().to_string(),
                })?;
        if suffix_segments.is_empty() {
            return Err(RefValidationError::MissingSuffix {
                value: parsed.as_str().to_string(),
            });
        }

        Ok((job_id, suffix_segments.join("/")))
    }
}

fn parse_ref(value: &str, expected_scheme: RefScheme) -> Result<ParsedRef, RefValidationError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(RefValidationError::EmptyValue);
    }

    let Some(remainder) = raw.strip_prefix(expected_scheme.prefix()) else {
        return Err(RefValidationError::InvalidScheme {
            expected: expected_scheme,
            actual: raw.to_string(),
        });
    };

    let segments = validate_segments(remainder)?;

    Ok(ParsedRef {
        raw: raw.to_string(),
        scheme: expected_scheme,
        segments,
    })
}

fn build_ref(scheme: RefScheme, segments: &[&str]) -> Result<String, RefValidationError> {
    if segments.is_empty() {
        return Err(RefValidationError::EmptyValue);
    }

    let mut normalized = Vec::with_capacity(segments.len());
    for segment in segments {
        if segment.is_empty() {
            return Err(RefValidationError::EmptySegment {
                value: scheme.prefix().to_string(),
            });
        }
        if !segment.is_ascii() {
            return Err(RefValidationError::NonAscii {
                value: segment.to_string(),
            });
        }
        if segment.contains('/') {
            return Err(RefValidationError::InvalidSegment {
                value: segment.to_string(),
            });
        }
        normalized.push(*segment);
    }

    Ok(format!("{}{}", scheme.prefix(), normalized.join("/")))
}

fn validate_segments(path: &str) -> Result<Vec<String>, RefValidationError> {
    if path.is_empty() {
        return Err(RefValidationError::EmptySegment {
            value: path.to_string(),
        });
    }

    if !path.is_ascii() {
        return Err(RefValidationError::NonAscii {
            value: path.to_string(),
        });
    }

    let mut segments = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() {
            return Err(RefValidationError::EmptySegment {
                value: path.to_string(),
            });
        }
        segments.push(segment.to_string());
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::{RefFactory, RefParser, RefValidationError};

    #[test]
    fn factory_builds_refs() {
        let input = RefFactory::input("cam", &["sample"]).unwrap();
        let result = RefFactory::result("cam", 42, &["ok"]).unwrap();
        let log = RefFactory::log("sim", 7, &["ok", "hybrid-gap-0.1"]).unwrap();

        assert_eq!(input, "input://cam/sample");
        assert_eq!(result, "result://cam/42/ok");
        assert_eq!(log, "log://sim/7/ok/hybrid-gap-0.1");
    }

    #[test]
    fn parser_rejects_non_ascii() {
        let result = RefParser::parse_input("input://cam/日本語");
        assert!(matches!(result, Err(RefValidationError::NonAscii { .. })));
    }

    #[test]
    fn parser_extracts_result_job_ref() {
        let (job_id, suffix) = RefParser::parse_result_job_ref("result://cam/9/ok", "cam").unwrap();
        assert_eq!(job_id, 9);
        assert_eq!(suffix, "ok");
    }

    #[test]
    fn parser_rejects_domain_mismatch() {
        let result = RefParser::parse_result_job_ref("result://sim/9/ok", "cam");
        assert!(matches!(
            result,
            Err(RefValidationError::DomainMismatch { .. })
        ));
    }
}
