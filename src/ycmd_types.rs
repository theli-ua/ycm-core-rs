pub struct Location {
    line_num: usize,
    column_num: usize,
    filepath: String,
}

pub struct Range {
    start: Location,
    end: Location,
}

pub struct FixitChunk {
    replacement_string: String,
    range: Range,
}

pub struct Fixit {
    text: String,
    location: Location,
    resolve: bool,
    kind: String,
    chunks: Vec<FixitChunk>,
}

pub struct CandidateExtraData {
    doc_string: String,
    fixits: Vec<Fixit>,
    resolve: Option<usize>,
}

pub struct Candidate {
    insertion_text: String,
    menu_text: Option<String>,
    extra_menu_info: String,
    detailed_info: String,
    kind: String,
    extra_data: CandidateExtraData,
}

#[allow(non_camel_case_types)]
pub enum CompleterTarget {
    filetype_default,
    identifier,
    filetype(String),
}

pub struct Exception {
    message: String,
}

pub struct ExceptionResponse {
    exception: Exception,
    message: String,
    traceback: String,
}

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

