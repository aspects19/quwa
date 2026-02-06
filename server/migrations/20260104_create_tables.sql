-- Users table (synced with Appwrite)
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    appwrite_id VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255),
    name VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Uploaded files metadata
CREATE TABLE IF NOT EXISTS uploaded_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_name VARCHAR(500) NOT NULL,
    file_type VARCHAR(50) NOT NULL, -- 'pdf', 'image'
    mime_type VARCHAR(100),
    file_size_bytes BIGINT,
    appwrite_file_id VARCHAR(255) NOT NULL,
    appwrite_bucket_id VARCHAR(255) NOT NULL,
    processing_status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed'
    upload_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT
);

-- Embeddings metadata (links vector store to files)
CREATE TABLE IF NOT EXISTS embeddings_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL REFERENCES uploaded_files(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    embedding_id VARCHAR(255) NOT NULL, -- Reference to vector store
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Orphadata diseases cache
CREATE TABLE IF NOT EXISTS orphadata_diseases (
    id SERIAL PRIMARY KEY,
    orpha_code VARCHAR(50) UNIQUE NOT NULL,
    disease_name TEXT NOT NULL,
    description TEXT,
    symptoms TEXT,
    diagnostic_criteria TEXT,
    prevalence VARCHAR(100),
    category VARCHAR(100),
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_uploaded_files_user_id ON uploaded_files(user_id);
CREATE INDEX IF NOT EXISTS idx_uploaded_files_status ON uploaded_files(processing_status);
CREATE INDEX IF NOT EXISTS idx_embeddings_file_id ON embeddings_metadata(file_id);
CREATE INDEX IF NOT EXISTS idx_orphadata_code ON orphadata_diseases(orpha_code);
