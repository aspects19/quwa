use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct RequestCounter {
    embedding_count: Arc<AtomicU64>,
    chat_count: Arc<AtomicU64>,
    start_time: Arc<AtomicU64>,
}

impl RequestCounter {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            embedding_count: Arc::new(AtomicU64::new(0)),
            chat_count: Arc::new(AtomicU64::new(0)),
            start_time: Arc::new(AtomicU64::new(now)),
        }
    }
    
    pub fn log_embedding_request(&self, context: &str) -> u64 {
        let count = self.embedding_count.fetch_add(1, Ordering::SeqCst) + 1;
        let elapsed = self.get_elapsed_seconds();
        
        tracing::info!(
            "🔢 GEMINI API REQUEST #{} | Type: EMBEDDING | Context: {} | Elapsed: {}s | Total Embeddings: {} | Total Chat: {} | Rate: {:.2} req/min",
            self.total_requests(),
            context,
            elapsed,
            count,
            self.get_chat_count(),
            self.get_request_rate()
        );
        
        count
    }
    
    pub fn log_chat_request(&self, context: &str) -> u64 {
        let count = self.chat_count.fetch_add(1, Ordering::SeqCst) + 1;
        let elapsed = self.get_elapsed_seconds();
        
        tracing::info!(
            "🔢 GEMINI API REQUEST #{} | Type: CHAT | Context: {} | Elapsed: {}s | Total Embeddings: {} | Total Chat: {} | Rate: {:.2} req/min",
            self.total_requests(),
            context,
            elapsed,
            self.get_embedding_count(),
            count,
            self.get_request_rate()
        );
        
        count
    }
    
    pub fn get_embedding_count(&self) -> u64 {
        self.embedding_count.load(Ordering::SeqCst)
    }
    
    pub fn get_chat_count(&self) -> u64 {
        self.chat_count.load(Ordering::SeqCst)
    }
    
    pub fn total_requests(&self) -> u64 {
        self.get_embedding_count() + self.get_chat_count()
    }
    
    fn get_elapsed_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let start = self.start_time.load(Ordering::SeqCst);
        now - start
    }
    
    fn get_request_rate(&self) -> f64 {
        let elapsed = self.get_elapsed_seconds() as f64;
        if elapsed < 1.0 {
            return 0.0;
        }
        (self.total_requests() as f64 / elapsed) * 60.0 // requests per minute
    }
    
    pub fn print_summary(&self) {
        tracing::info!(
            "📊 GEMINI API SUMMARY | Total: {} requests | Embeddings: {} | Chat: {} | Elapsed: {}s | Avg Rate: {:.2} req/min",
            self.total_requests(),
            self.get_embedding_count(),
            self.get_chat_count(),
            self.get_elapsed_seconds(),
            self.get_request_rate()
        );
    }
}

impl Default for RequestCounter {
    fn default() -> Self {
        Self::new()
    }
}
