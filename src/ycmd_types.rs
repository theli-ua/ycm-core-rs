#![allow(dead_code)]

use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct Location {
    line_num: usize,
    column_num: usize,
    filepath: String,
}

#[derive(serde::Deserialize)]
pub struct FileData {
    filetypes: Vec<String>,
    contents: String,
}

#[derive(serde::Deserialize)]
pub struct SimpleRequest {
    line_num: usize,
    column_num: usize,
    filepath: String,
    file_data: HashMap<String, FileData>,
    completer_target: CompleterTarget,
    working_dir: String,
    extra_conf_data: serde_json::Value,
}

#[derive(serde::Serialize)]
pub struct Range {
    start: Location,
    end: Location,
}

#[derive(serde::Serialize)]
pub struct FixitChunk {
    replacement_string: String,
    range: Range,
}

#[derive(serde::Serialize)]
pub struct Fixit {
    text: String,
    location: Location,
    resolve: bool,
    kind: String,
    chunks: Vec<FixitChunk>,
}

#[derive(serde::Serialize)]
pub struct CandidateExtraData {
    doc_string: String,
    fixits: Vec<Fixit>,
    resolve: Option<usize>,
}

#[derive(serde::Serialize)]
pub struct Candidate {
    insertion_text: String,
    menu_text: Option<String>,
    extra_menu_info: String,
    detailed_info: String,
    kind: String,
    extra_data: CandidateExtraData,
}

#[allow(non_camel_case_types)]
#[derive(serde::Deserialize)]
pub enum CompleterTarget {
    filetype_default,
    identifier,
    filetype(String),
}

#[derive(serde::Serialize)]
pub struct Exception {
    message: String,
}

#[derive(serde::Serialize)]
pub struct ExceptionResponse {
    exception: Exception,
    message: String,
    traceback: String,
}

#[derive(serde::Serialize)]
pub struct CompletionResponse {
    completions: Vec<Candidate>,
    completion_start_column: usize,
    errors: Vec<ExceptionResponse>,
}

pub struct ItemData {
    key: String,
    value: String,
}

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
pub struct DebugInfoResponse {
    name: String,
    servers: Vec<ServerData>,
    items: Vec<ItemData>,
}

pub enum DiagnosticKind {
    WARNING,
    ERROR,
    INFORMATION,
    HINT,
}

pub struct DiagnosticData {
    ranges: Vec<Range>,
    location: Location,
    location_extent: Range,
    test: String,
    kind: DiagnosticKind,
    fixit_available: bool,
}

pub struct DiagnosticMessage {
    filepath: String,
    diagnostics: Vec<DiagnosticData>,
}

