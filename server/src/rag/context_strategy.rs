
use chrono::{DateTime, Utc};

const GEMINI_CONTEXT_WINDOW: usize = 1_000_000; // 1M tokens
const LARGE_DOC_THRESHOLD: usize = 50_000; // characters
const CRITICAL_FILE_TYPES: &[&str] = &["pdf", "lab_report"];

#[derive(Debug, Clone)]
pub struct ContextStrategy {
    pub use_full_context: bool,
    pub rag_top_k: usize,
    pub include_full_docs: Vec<String>,
}

pub fn determine_strategy(
    user_files: &[crate::db::models::UploadedFile],
    _query: &str,
) -> ContextStrategy {
    let recent_critical_files: Vec<_> = user_files
        .iter()
        .filter(|f| CRITICAL_FILE_TYPES.contains(&f.file_type.as_str()))
        .filter(|f| is_recent(f.upload_date))
        .map(|f| f.id.to_string())
        .collect();
    
    // If user has critical recent files, prioritize full-context for those
    if !recent_critical_files.is_empty() {
        ContextStrategy {
            use_full_context: true,
            rag_top_k: 3,
            include_full_docs: recent_critical_files,
        }
    } else {
        // Default to RAG-only
        ContextStrategy {
            use_full_context: false,
            rag_top_k: 5,
            include_full_docs: vec![],
        }
    }
}

fn is_recent(upload_date: DateTime<Utc>) -> bool {
    let now = Utc::now();
    let duration = now.signed_duration_since(upload_date);
    duration.num_days() <= 7 // Files from last 7 days
}
