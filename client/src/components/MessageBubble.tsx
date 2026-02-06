interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  thinking?: string[]
}

interface MessageBubbleProps {
  message: Message
}

export default function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === 'user'
  
  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'} mb-4`}>
      <div
        className={`max-w-[80%] rounded-2xl px-5 py-3 ${
          isUser
            ? 'gradient-primary text-white'
            : 'glass-strong border border-white/10'
        }`}
      >
        {!isUser && (
          <div className="text-xs font-semibold text-primary-400 mb-2">
            Quwa AI
          </div>
        )}
        <div className={`prose prose-invert ${isUser ? 'prose-p:text-white' : ''}`}>
          <p className="whitespace-pre-wrap leading-relaxed">
            {message.content}
          </p>
        </div>
      </div>
    </div>
  )
}
