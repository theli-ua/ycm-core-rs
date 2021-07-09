#![allow(dead_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Location {
    line_num: usize,
    column_num: usize,
    filepath: String,
}

#[derive(Deserialize, Debug)]
pub struct FileData {
    filetypes: Vec<String>,
    contents: String,
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
    trigger: String,
    description: String,
}

#[derive(Deserialize, Debug)]
pub struct EventNotification {
    line_num: usize,
    column_num: usize,
    filepath: String,
    file_data: HashMap<String, FileData>,
    completer_target: Option<CompleterTarget>,
    working_dir: Option<String>,
    extra_conf_data: Option<serde_json::Value>,
    event_name: Event,
    ultisnips_snippets: Option<Vec<UltisnipSnippet>>,
}

#[derive(Deserialize)]
pub struct SimpleRequest {
    line_num: usize,
    column_num: usize,
    filepath: String,
    file_data: HashMap<String, FileData>,
    completer_target: Option<CompleterTarget>,
    working_dir: Option<String>,
    extra_conf_data: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct Range {
    start: Location,
    end: Location,
}

#[derive(Serialize)]
pub struct FixitChunk {
    replacement_string: String,
    range: Range,
}

#[derive(Serialize)]
pub struct Fixit {
    text: String,
    location: Location,
    resolve: bool,
    kind: String,
    chunks: Vec<FixitChunk>,
}

#[derive(Serialize)]
pub struct CandidateExtraData {
    doc_string: String,
    fixits: Vec<Fixit>,
    resolve: Option<usize>,
}

#[derive(Serialize)]
pub struct Candidate {
    insertion_text: String,
    menu_text: Option<String>,
    extra_menu_info: String,
    detailed_info: String,
    kind: String,
    extra_data: CandidateExtraData,
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
    completions: Vec<Candidate>,
    completion_start_column: usize,
    errors: Vec<ExceptionResponse>,
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
    name: String,
    servers: Vec<ServerData>,
    items: Vec<ItemData>,
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

