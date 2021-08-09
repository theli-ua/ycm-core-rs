#![allow(dead_code)]

use std::{collections::HashMap, path::PathBuf, str::Lines};

use serde::{Deserialize, Serialize};

use crate::core::utils::identifier::start_of_longest_identifier_ending_at_index;

#[derive(Serialize, Clone, Debug)]
pub struct Location {
    line_num: usize,
    column_num: usize,
    filepath: String,
}

#[derive(Deserialize, Debug)]
pub struct FileData {
    pub filetypes: Vec<String>,
    pub contents: String,
}

#[derive(Deserialize, Debug)]
pub enum Event {
    FileReadyToParse,
    BufferUnload,
    BufferVisit,
    InsertLeave,
    CurrentIdentifierFinished,
}

#[derive(Deserialize, Debug)]
pub struct UltisnipSnippet {
    pub trigger: String,
    pub description: String,
}

#[derive(Deserialize, Debug)]
pub struct EventNotification {
    pub line_num: usize,
    pub column_num: usize,
    pub filepath: String,
    pub file_data: HashMap<String, FileData>,
    pub completer_target: Option<CompleterTarget>,
    pub working_dir: Option<String>,
    pub extra_conf_data: Option<serde_json::Value>,
    pub event_name: Event,
    pub ultisnips_snippets: Option<Vec<UltisnipSnippet>>,
}

#[derive(Deserialize, Debug)]
pub struct SimpleRequest {
    /// 1-based line number
    pub line_num: usize,
    /// 1-based byte offset
    pub column_num: usize,
    pub filepath: PathBuf,
    pub file_data: HashMap<PathBuf, FileData>,
    pub completer_target: Option<CompleterTarget>,
    pub working_dir: Option<PathBuf>,
    pub extra_conf_data: Option<serde_json::Value>,
}

impl SimpleRequest {
    pub fn lines(&self) -> Lines {
        self.file_data.get(&self.filepath).unwrap().contents.lines()
    }

    pub fn filetypes(&self) -> &[String] {
        match self.file_data.get(&self.filepath) {
            Some(f) => &f.filetypes,
            None => &[],
        }
    }

    pub fn first_filetype(&self) -> Option<&str> {
        self.filetypes().get(0).map(String::as_str)
    }

    /// current line
    pub fn line_value(&self) -> &str {
        self.lines().nth(self.line_num - 1).unwrap()
    }

    /// The calculated start column, as a byte offset into the UTF-8 encoded
    /// bytes returned by line_bytes
    pub fn start_column(&self) -> usize {
        start_of_longest_identifier_ending_at_index(
            self.line_value(),
            self.column_num - 1,
            self.first_filetype(),
        )
    }

    /// 'query' after the beginning
    /// of the identifier to be completed
    pub fn query(&self) -> &str {
        &self.line_value()[self.start_column()..=self.column_num - 2]
    }

    /// line value up to the character
    /// before the start of 'query'
    pub fn prefix(&self) -> &str {
        let start = self.start_column();
        if start == 0 {
            ""
        } else {
            &self.line_value()[..=self.start_column() - 1]
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Range {
    start: Location,
    end: Location,
}

#[derive(Serialize, Clone, Debug)]
pub struct FixitChunk {
    replacement_string: String,
    range: Range,
}

#[derive(Serialize, Clone, Debug)]
pub struct Fixit {
    text: String,
    location: Location,
    resolve: bool,
    kind: String,
    chunks: Vec<FixitChunk>,
}

#[derive(Serialize, Clone, Debug)]
pub struct CandidateExtraData {
    doc_string: String,
    fixits: Vec<Fixit>,
    resolve: Option<usize>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FilterAndSortRequest {
    pub candidates: Vec<serde_json::Value>,
    pub sort_property: String,
    pub query: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Candidate {
    pub insertion_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_menu_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_data: Option<CandidateExtraData>,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
pub enum CompleterTarget {
    filetype_default,
    identifier,
    filetype(String),
}

#[derive(Serialize)]
pub struct Exception {
    message: String,
}

#[derive(Serialize)]
pub struct ExceptionResponse {
    exception: Exception,
    message: String,
    traceback: String,
}

#[derive(Serialize)]
pub struct CompletionResponse {
    pub completions: Vec<Candidate>,
    pub completion_start_column: usize,
    pub errors: Vec<ExceptionResponse>,
}

#[derive(Serialize)]
pub struct ItemData {
    key: String,
    value: String,
}

#[derive(Serialize)]
pub struct ServerData {
    name: String,
    is_running: bool,
    executable: String,
    address: String,
    port: usize,
    pid: usize,
    logfiles: Vec<String>,
    extras: Vec<ItemData>,
}

#[derive(Serialize)]
pub struct DebugInfoResponse {
    pub name: String,
    pub servers: Vec<ServerData>,
    pub items: Vec<ItemData>,
}

#[derive(Serialize)]
pub struct PythonInfo {
    pub executable: String,
    pub version: String,
}
#[derive(Serialize)]
pub struct ClangInfo {
    pub has_support: bool,
    pub version: Option<String>,
}
#[derive(Serialize)]
pub struct ExtraInfo {
    pub path: String,
    pub is_loaded: bool,
}
#[derive(Serialize)]
pub struct DebugInfo {
    pub python: PythonInfo,
    pub clang: ClangInfo,
    pub extra_conf: ExtraInfo,
    pub completer: DebugInfoResponse,
}

#[derive(Serialize)]
pub enum DiagnosticKind {
    WARNING,
    ERROR,
    INFORMATION,
    HINT,
}

#[derive(Serialize)]
pub struct DiagnosticData {
    ranges: Vec<Range>,
    location: Location,
    location_extent: Range,
    test: String,
    kind: DiagnosticKind,
    fixit_available: bool,
}

#[derive(Serialize)]
pub struct DiagnosticMessage {
    filepath: String,
    diagnostics: Vec<DiagnosticData>,
}

#[derive(Serialize)]
pub enum Available {
    YES,
    NO,
    PENDING,
}

#[derive(Deserialize)]
pub struct Subserver {
    subserver: String,
}

#[derive(Serialize)]
pub struct SimpleMessage {
    message: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Message {
    SimpleMessage(SimpleMessage),
    Diagnostics(DiagnosticMessage),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum MessagePollResponse {
    MessagePollResponse(bool),
    Message(Message),
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    fn get_simple_request<S: ToString, P: AsRef<Path>>(
        file_contents: S,
        filepath: P,
        line_num: usize,
        column_num: usize,
    ) -> SimpleRequest {
        let mut file_data = std::collections::HashMap::default();

        let filepath = filepath.as_ref().to_path_buf();

        file_data.insert(
            filepath.clone(),
            FileData {
                filetypes: vec![String::from("rust"), String::from("c")],
                contents: file_contents.to_string(),
            },
        );
        SimpleRequest {
            line_num,
            column_num,
            filepath,
            file_data,
            completer_target: None,
            working_dir: None,
            extra_conf_data: None,
        }
    }

    #[test]
    fn simple_request_lines() {
        let request = get_simple_request("a\nb\n\n\nc", "aa", 0, 0);
        assert_eq!(
            request.lines().collect::<Vec<_>>(),
            vec!["a", "b", "", "", "c"]
        );
    }

    #[test]
    fn simple_request_line_value() {
        let request = get_simple_request("a\nb\n\n\nc", "aa", 2, 0);
        assert_eq!(request.line_value(), "b");
    }

    #[test]
    fn simple_request_filetypes() {
        let request = get_simple_request("a\nb\n\n\nc", "aa", 2, 0);
        assert_eq!(
            request.filetypes(),
            vec![String::from("rust"), String::from("c")]
        );
    }

    #[test]
    fn simple_request_first_filetype() {
        let request = get_simple_request("a\nb\n\n\nc", "aa", 2, 0);
        assert_eq!(request.first_filetype(), Some("rust"));
    }

    #[test]
    fn simple_request_start_column() {
        let request = get_simple_request("12345 a8", "aa", 1, 9);
        assert_eq!(request.start_column(), 6);

        let request = get_simple_request("u", "aa", 1, 2);
        assert_eq!(request.start_column(), 0);
    }

    #[test]
    fn simple_request_query() {
        let request = get_simple_request("12345 a8", "aa", 1, 9);
        assert_eq!(request.query(), "a8");
        let request = get_simple_request("u", "aa", 1, 2);
        assert_eq!(request.query(), "u");
    }

    #[test]
    fn simple_request_prefix() {
        let request = get_simple_request("12345 a8", "aa", 1, 9);
        assert_eq!(request.prefix(), "12345 ");

        let request = get_simple_request("unim", "aa", 1, 5);
        assert_eq!(request.prefix(), "");
    }
}
