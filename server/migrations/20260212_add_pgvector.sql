-- Enable pgvector extension for vector similarity search
CREATE EXTENSION IF NOT EXISTS vector;

-- Embeddings table with pgvector support
CREATE TABLE IF NOT EXISTS embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    text TEXT NOT NULL,
    embedding vector(384), -- FastEmbed all-MiniLM-L6-v2 produces 384-dimensional vectors
    source_type VARCHAR(50) NOT NULL, -- 'orphadata', 'user_file', etc.
    source_id VARCHAR(255) NOT NULL,
    file_name VARCHAR(500),
    orpha_code VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for vector similarity search using HNSW (Hierarchical Navigable Small World)
-- This provides fast approximate nearest neighbor search
CREATE INDEX IF NOT EXISTS embeddings_embedding_idx ON embeddings 
USING hnsw (embedding vector_cosine_ops);

-- Indexes for filtering
CREATE INDEX IF NOT EXISTS idx_embeddings_source_type ON embeddings(source_type);
CREATE INDEX IF NOT EXISTS idx_embeddings_source_id ON embeddings(source_id);
CREATE INDEX IF NOT EXISTS idx_embeddings_orpha_code ON embeddings(orpha_code);
