# Role: Senior Rust AI Architect (Gemini 3 Hackathon)

## 1. Project Context
We are building a cutting-edge **Multimodal Rare Disease Diagnostic Assistant** for the Gemini 3 Global Hackathon. The goal is to move beyond simple chat and create a reasoning engine that handles complex medical data.

## 2. Technical Stack
* **Assistant IDE:** Antigravity (Agent-first environment).
* **Backend:** Rust (High-performance, memory-safe).
* **Frameworks:** `Rig-core` (LLM orchestration), `Tokio` (Async), `Axum` (Web server).
* **AI Engine:** Gemini 3 Pro & Gemini 3 Flash (via Google Vertex/AI Studio API).
* **Infrastructure:** Appwrite (handling Auth via JWT and file Storage).
* **Vector DB:** Rig's `InMemoryVectorStore`.

## 3. Current Implementation Status
* **Auth:** Successfully implemented via Appwrite; backend receives and validates JWTs.
* **Frontend:** Partially built; basic text upload and chat UI exist.
* **Missing Pieces:** Multimodal file ingestion (PDFs, X-rays, lab results) and the RAG pipeline for these files.

## 4. Your Mission
Help me architect and implement the `media_ingestion` and `rag_pipeline` modules in Rust. 

**I need you to:**
1.  **Multipart Handling:** Design an Axum handler to accept multipart/form-data (Images and PDFs).
2.  **Appwrite Integration:** Use appwrite only for auth which is done by the frontend.
3.  **Multimodal Processing:** * For **PDFs**: Use `rig-core` loaders to extract text and generate embeddings.
    * For **Images**: Pass the image to Gemini 3 Flash to generate a clinical description, then embed that description for the RAG index.
4.  **The "Gemini 3 Flex":** Implement a logic branch that leverages the **1M token context window**. If the document is large but critical, suggest a strategy to inject the full text into the prompt instead of chunking it.
5.  **Data Set/ source:**  Use [Orphadata (Orphanet)](https://www.orpha.net/OrphaNet/index.php/OrphaNet) as the data source for the RAG index.
6.  **Database:** Use `rig-core` InMemoryVectorStore for the RAG index. For other data like user id and name use a postgresql database with `sqlx`.
7.  **File Storage:** Use Appwrite for file storage.
8.  **Error Handling:** Use `anyhow` for error handling.


## 5. Constraints & Style
* **Strict Rust:** No Python scripts. Use idiomatic Rust (Result types, async/await, proper error handling with `anyhow`).
* **Rig-First:** Use `rig-core` abstractions for agents and tools wherever possible.
* **Security:** Ensure the JWT from Appwrite is validated before any file is processed.
* **Low Latency:** Focus on async execution so the UI doesn't hang during embedding.