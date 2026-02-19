import { useRef, useState, useEffect } from "react";
import { Send, Loader2, FileText, Image as ImageIcon, X, Paperclip, Plus } from "lucide-react";
import ThinkingProcess from "./ThinkingProcess";
import SourceCitation from "./SourceCitation";
import { getValidJWT } from "@/lib/appwrite";
import {SSE} from 'sse.js';
const BACKEND_URL = import.meta.env.VITE_BACKEND_URL || 'http://localhost:3000';

interface UploadedFile {
  id: string;
  name: string;
  status: 'uploading' | 'processing' | 'completed' | 'failed';
  error?: string;
}

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  thinking?: string[];
  sources?: Array<{
    source_type: string;
    source_id: string;
    relevance: number;
  }>;
}
export default function ChatInterface() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [uploadedFiles, setUploadedFiles] = useState<UploadedFile[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const inputRef = useRef<HTMLDivElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const sseSourceRef = useRef<SSE | null>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  useEffect(() => {
    return () => {
      if (sseSourceRef.current) {
        sseSourceRef.current.close();
      }
    };
  }, []);
  const handleInput = () => {
    if (!inputRef.current) return;
    setInput(inputRef.current.innerText);
  };
  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as any);
    }
  };

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files) return;

    Array.from(files).forEach(file => {
      const error = validateFile(file);
      if (error) {
        setUploadedFiles(prev => [...prev, {
          id: `${Date.now()}-${file.name}`,
          name: file.name,
          status: 'failed',
          error,
        }]);
      } else {
        uploadFile(file);
      }
    });

    // Reset input
    e.target.value = '';
  };

  const validateFile = (file: File): string | null => {
    const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB
    const ALLOWED_EXTENSIONS = ['pdf', 'jpg', 'jpeg', 'png'];

    if (file.size > MAX_FILE_SIZE) {
      return 'File size exceeds 50MB limit';
    }

    const extension = file.name.split('.').pop()?.toLowerCase();
    if (!extension || !ALLOWED_EXTENSIONS.includes(extension)) {
      return 'Only PDF and image files (JPG, PNG) are allowed';
    }

    return null;
  };

  const uploadFile = async (file: File) => {
    const fileId = `${Date.now()}-${file.name}`;
    
    setUploadedFiles(prev => [...prev, {
      id: fileId,
      name: file.name,
      status: 'uploading',
    }]);

    try {
      const jwtToken = await getValidJWT();
      const formData = new FormData();
      formData.append('file', file);

      const response = await fetch(`${BACKEND_URL}/api/upload`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${jwtToken}`,
        },
        body: formData,
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || 'Upload failed');
      }

      await response.json();
      
      setUploadedFiles(prev => prev.map(f => 
        f.id === fileId 
          ? { ...f, status: 'processing' }
          : f
      ));

      setTimeout(() => {
        setUploadedFiles(prev => prev.map(f => 
          f.id === fileId 
            ? { ...f, status: 'completed' }
            : f
        ));
      }, 2000);

    } catch (error) {
      console.error('Upload error:', error);
      setUploadedFiles(prev => prev.map(f => 
        f.id === fileId 
          ? { 
              ...f, 
              status: 'failed', 
              error: error instanceof Error ? error.message : 'Upload failed'
            }
          : f
      ));
    }
  };

  const removeFile = (id: string) => {
    setUploadedFiles(prev => prev.filter(f => f.id !== id));
  };

  const getFileIcon = (fileName: string) => {
    const ext = fileName.split('.').pop()?.toLowerCase();
    if (ext === 'pdf') return FileText;
    return ImageIcon;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;
    const userMessage: Message = {
      id: Date.now().toString(),
      role: "user",
      content: input.trim(),
    };
    setMessages((prev) => [...prev, userMessage]);
    const messageContent = input.trim();
    setInput("");
    setIsLoading(true);
    if (inputRef.current) {
      inputRef.current.innerHTML = "";
    }
    try {
      const jwtToken = await getValidJWT();
      const source = new SSE(`${BACKEND_URL}/api/chat`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${jwtToken}`,
        },
        payload: JSON.stringify({
          message: messageContent,
        }),
      });

      sseSourceRef.current = source;
      const assistantMessageId = (Date.now() + 1).toString();
      let accumulatedContent = "";
      const thinkingSteps: string[] = [];
      //prevents a false error that is returned after the done state is returned
      let isDone = false;
      
      const initialAssistantMessage: Message = {
        id: assistantMessageId,
        role: "assistant",
        content: "",
        thinking: [],
        sources: [],
      };
      setMessages((prev) => [...prev, initialAssistantMessage]);
      
      source.addEventListener('thinking', (e: any) => {
        const data = JSON.parse(e.data);
        const nextStep = String(data.step || '').trim();
        if (!nextStep) return;

        if (thinkingSteps[thinkingSteps.length - 1] !== nextStep) {
          thinkingSteps.push(nextStep);
        }
        
        setMessages((prev) => 
          prev.map((msg) => 
            msg.id === assistantMessageId 
              ? { ...msg, thinking: [...thinkingSteps] }
              : msg
          )
        );
      });
      source.addEventListener('response', (e: any) => {
        const data = JSON.parse(e.data);
        accumulatedContent += data.content;
      
        setMessages((prev) => 
          prev.map((msg) => 
            msg.id === assistantMessageId 
              ? { ...msg, content: accumulatedContent, thinking: [] }
              : msg
          )
        );
      });

      source.addEventListener('source', (e: any) => {
        const data = JSON.parse(e.data);
        
        setMessages((prev) => 
          prev.map((msg) => 
            msg.id === assistantMessageId 
              ? { ...msg, sources: [...(msg.sources || []), data] }
              : msg
          )
        );
      });

      source.addEventListener('done', () => {
        setIsLoading(false);
        isDone = true;
        source.close();
        sseSourceRef.current = null;
      });
      
      source.addEventListener('error', () => {
        if (isDone) {
          return;
        }
        setIsLoading(false);
        
        if (source) {
          source.close();
        }
        sseSourceRef.current = null;
        
        setMessages((prev) => 
          prev.map((msg) => 
            msg.id === assistantMessageId && !msg.content
              ? { ...msg, content: "Failed to respond correctly. Try again", thinking: [] }
              : msg
          )
        );
      });

      source.stream();
      
    } catch (err) {
      console.error('Error sending message:', err);
      setIsLoading(false);
      sseSourceRef.current = null;
    }
  };
  return (
    <div className="w-full max-w-5xl h-[calc(100vh-120px)] mx-auto flex flex-col gap-4">
      {/* Messages Container */}
      <div className="flex-1 overflow-y-auto space-y-4 p-6 card">
        {messages.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-center px-4">
            <h2 className="text-2xl font-semibold text-white mb-3">
              Rare Disease Assistant
            </h2>
            <p className="max-w-md text-lg leading-relaxed text-white/70">
              Describe patient symptoms or clinical observations to receive
              AI-assisted rare disease analysis.
            </p>
            <div className="mt-8 w-full max-w-2xl">
              <form
                onSubmit={handleSubmit}
                className="rounded-2xl pb-2 border border-white/10 bg-white/5 backdrop-blur-md shadow-lg transition focus-within:border-info/20 focus-within:shadow-primary/10"
              >
                {/* Uploaded Files Display */}
                {uploadedFiles.length > 0 && (
                  <div className="px-4 pt-3 pb-2 border-b border-white/10 space-y-1">
                    {uploadedFiles.map((file) => {
                      const FileIcon = getFileIcon(file.name);
                      return (
                        <div
                          key={file.id}
                          className="flex items-center gap-2 px-2 py-1.5 rounded-lg bg-white/5 text-sm"
                        >
                          <FileIcon className="w-3.5 h-3.5 text-white/60 shrink-0" />
                          <span className="flex-1 text-white/80 truncate text-xs">{file.name}</span>
                          <span className={`text-xs ${
                            file.status === 'completed' ? 'text-green-400' :
                            file.status === 'failed' ? 'text-red-400' :
                            'text-blue-400'
                          }`}>
                            {file.status}
                          </span>
                          <button
                            type="button"
                            onClick={() => removeFile(file.id)}
                            className="p-0.5 rounded hover:bg-white/10"
                          >
                            <X className="w-3.5 h-3.5 text-white/60" />
                          </button>
                        </div>
                      );
                    })}
                  </div>
                )}
                
                <div className="flex text-start gap-3 p-4 py-2">
                  <input
                    type="file"
                    id="file-upload-initial"
                    multiple
                    accept=".pdf,.jpg,.jpeg,.png"
                    onChange={handleFileSelect}
                    className="hidden"
                  />
                  <label
                    htmlFor="file-upload-initial"
                    className="flex place-self-end h-10 w-10 items-center justify-center rounded-full hover:bg-primary/20 transition-all cursor-pointer group hover:scale-110 active:scale-95"
                  >
                    <Plus className="h-6 w-6 font-bold text-white/70 group-hover:text-primary transition-all group-hover:rotate-90" />
                  </label>
                  <div
                    ref={inputRef}
                    contentEditable={!isLoading}
                    role="textbox"
                    aria-multiline="true"
                    data-placeholder="Describe patient symptoms or clinical observations..."
                    className="flex-1 min-h-6 max-h-[40vh] overflow-y-auto bg-transparent text-base leading-relaxed text-white outline-none whitespace-pre-wrap wrap-break-word empty:before:text-white/40 empty:before:pointer-events-none selection:bg-primary/30"
                    onInput={handleInput}
                    onKeyDown={handleKeyDown}
                  />
                  <button
                    type="submit"
                    disabled={!input.trim() || isLoading}
                    className="flex place-self-end h-10 w-10 items-center justify-center rounded-full bg-primary text-white transition-all hover:scale-110 hover:shadow-lg hover:shadow-primary/50 active:scale-95 disabled:bg-primary/30 disabled:hover:scale-100 disabled:shadow-none"
                  >
                    {isLoading ? (
                      <Loader2 className="h-5 w-5 animate-spin" />
                    ) : (
                      <Send className="h-5 w-5" />
                    )}
                  </button>
                </div>
              </form>
            </div>
          </div>
        ) : (
          <>
            {messages.map((message) => (
              <div key={message.id} className="animate-fade-in space-y-3">
                {message.thinking && message.role === "assistant" && message.thinking.length > 0 && (
                  <ThinkingProcess steps={message.thinking} />
                )}
                
                <div className={`flex gap-3 ${message.role === "user" ? "justify-end" : "justify-start"}`}>
                  <div className={`max-w-[80%] rounded-2xl px-5 py-3 ${
                    message.role === "user" 
                      ? "bg-primary text-white" 
                      : "bg-white/10 border border-white/10 text-white backdrop-blur-md"
                  }`}>
                    {message.role === "assistant" && (
                      <div className="text-xs font-medium text-white/60 mb-2">AI Assistant</div>
                    )}
                    <div className="text-base leading-relaxed whitespace-pre-wrap">
                      {message.content}
                      {message.role === "assistant" && !message.content && isLoading && (
                        <span className="inline-flex items-center gap-1 text-white/50">
                          <span className="animate-pulse">●</span>
                          <span className="animate-pulse animation-delay-200">●</span>
                          <span className="animate-pulse animation-delay-400">●</span>
                        </span>
                      )}
                    </div>
                    {message.role === "assistant" && message.sources && message.sources.length > 0 && (
                      <SourceCitation sources={message.sources} />
                    )}
                  </div>
                </div>
              </div>
            ))}
            {isLoading && messages[messages.length - 1]?.role !== "assistant" && (
              <div className="flex items-center gap-2 text-white/50 animate-pulse">
                <Loader2 className="w-4 h-4 animate-spin" />
                <span className="text-sm">Analyzing…</span>
              </div>
            )}
            
            {/* Scroll anchor */}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>
      {/* Input Section - Always visible after first message */}
      {messages.length > 0 && (
        <form
          onSubmit={handleSubmit}
          className="rounded-2xl pb-2 border border-white/10 bg-white/5 backdrop-blur-md shadow-lg transition focus-within:border-info/20 focus-within:shadow-primary/10"
        >
          {/* Uploaded Files Display */}
          {uploadedFiles.length > 0 && (
            <div className="px-4 pt-3 pb-2 border-b border-white/10 space-y-1">
              {uploadedFiles.map((file) => {
                const FileIcon = getFileIcon(file.name);
                return (
                  <div
                    key={file.id}
                    className="flex items-center gap-2 px-2 py-1.5 rounded-lg bg-white/5 text-sm"
                  >
                    <FileIcon className="w-3.5 h-3.5 text-white/60 shrink-0" />
                    <span className="flex-1 text-white/80 truncate text-xs">{file.name}</span>
                    <span className={`text-xs ${
                      file.status === 'completed' ? 'text-green-400' :
                      file.status === 'failed' ? 'text-red-400' :
                      'text-blue-400'
                    }`}>
                      {file.status}
                    </span>
                    <button
                      type="button"
                      onClick={() => removeFile(file.id)}
                      className="p-0.5 rounded hover:bg-white/10"
                    >
                      <X className="w-3.5 h-3.5 text-white/60" />
                    </button>
                  </div>
                );
              })}
            </div>
          )}
          
          <div className="flex text-start gap-3 p-4 py-2">
            <input
              type="file"
              id="file-upload-ongoing"
              multiple
              accept=".pdf,.jpg,.jpeg,.png"
              onChange={handleFileSelect}
              className="hidden"
            />
            <label
              htmlFor="file-upload-ongoing"
              className="flex place-self-end h-10 w-10 items-center justify-center rounded-full hover:bg-primary/20 transition-all cursor-pointer group hover:scale-110 active:scale-95"
            >
              <Paperclip className="h-5 w-5 text-white/70 group-hover:text-primary transition-all group-hover:rotate-12" />
            </label>
            <div
              ref={inputRef}
              contentEditable={!isLoading}
              role="textbox"
              aria-multiline="true"
              data-placeholder="Continue the conversation..."
              className="flex-1 min-h-6 max-h-[40vh] overflow-y-auto bg-transparent text-base leading-relaxed text-white outline-none whitespace-pre-wrap wrap-break-word empty:before:text-white/40 empty:before:pointer-events-none selection:bg-primary/30"
              onInput={handleInput}
              onKeyDown={handleKeyDown}
            />
            <button
              type="submit"
              disabled={!input.trim() || isLoading}
              className="flex place-self-end h-10 w-10 items-center justify-center rounded-full bg-primary text-white transition-all hover:scale-110 hover:shadow-lg hover:shadow-primary/50 active:scale-95 disabled:bg-primary/30 disabled:cursor-not-allowed disabled:hover:scale-100 disabled:shadow-none"
            >
              {isLoading ? (
                <Loader2 className="h-5 w-5 animate-spin" />
              ) : (
                <Send className="h-5 w-5" />
              )}
            </button>
          </div>
        </form>
      )}
    </div>
  );
}
