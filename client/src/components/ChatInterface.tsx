import { useRef, useState, useEffect } from "react";
import { Send, Loader2 } from "lucide-react";
import ThinkingProcess from "./ThinkingProcess";
import { getValidJWT } from "@/lib/appwrite";
import {SSE} from 'sse.js';
const BACKEND_URL = import.meta.env.VITE_BACKEND_URL || 'http://localhost:3000';
interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  thinking?: string[];
}
export default function ChatInterface() {
  const [messages, setMessages] = useState<Message[]>([]);
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
      let currentThinkingStep = "";
      //prevents a false error that is returned after the done state is returned
      let isDone = false;
      
      const initialAssistantMessage: Message = {
        id: assistantMessageId,
        role: "assistant",
        content: "",
        thinking: [],
      };
      setMessages((prev) => [...prev, initialAssistantMessage]);
      
      source.addEventListener('thinking', (e: any) => {
        const data = JSON.parse(e.data);
        currentThinkingStep = data.step;
        
        setMessages((prev) => 
          prev.map((msg) => 
            msg.id === assistantMessageId 
              ? { ...msg, thinking: [currentThinkingStep] }
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

      source.addEventListener('done', (e: any) => {
        setIsLoading(false);
        isDone = true;
        source.close();
        sseSourceRef.current = null;
      });
      
      source.addEventListener('error', (e: any) => {
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
                <div className="flex text-start gap-3 p-4 py-2">
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
                    className="flex place-self-end h-10 w-10 items-center justify-center rounded-full bg-primary text-white transition-all hover:scale-105 hover:bg-primary/90 active:scale-95 disabled:bg-primary/30 disabled:hover:scale-100"
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
          <div className="flex text-start gap-3 p-4 py-2">
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
              className="flex place-self-end h-10 w-10 items-center justify-center rounded-full bg-primary text-white transition-all hover:scale-105 hover:bg-primary/90 active:scale-95 disabled:bg-primary/30 disabled:cursor-not-allowed disabled:hover:scale-100"
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