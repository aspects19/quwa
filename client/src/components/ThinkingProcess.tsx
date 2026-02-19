import { useState } from 'react'
import { ChevronDown, ChevronUp } from 'lucide-react'

interface ThinkingProcessProps {
  steps: string[]
}

export default function ThinkingProcess({ steps }: ThinkingProcessProps) {
  const [isExpanded, setIsExpanded] = useState(true)
  
  if (steps.length === 0) return null
  
  return (
    <div className="ml-10 mb-3 rounded-lg border border-white/5 bg-white/5 px-3 py-2 text-xs text-white/50 backdrop-blur-sm animate-slide-in">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="flex items-center justify-between w-full text-left group"
      >
        <div className="flex items-center gap-2">
          <span className="inline-block h-2 w-2 rounded-full bg-white/40 animate-pulse" />
          <span className="text-[11px] uppercase tracking-wider text-white/40">
            Thinking
          </span>
          {steps.length > 1 && (
            <span className="text-[11px] text-white/30">
              ({steps.length} steps)
            </span>
          )}
        </div>
        {isExpanded ? (
          <ChevronUp className="w-4 h-4 text-white/30 group-hover:text-white/60 transition-colors" />
        ) : (
          <ChevronDown className="w-4 h-4 text-white/30 group-hover:text-white/60 transition-colors" />
        )}
      </button>
      
      {isExpanded && (
        <div className="mt-2 space-y-1.5">
          {steps.map((step, index) => (
            <div
              key={index}
              className="flex items-start gap-2 rounded-md bg-white/5 px-2 py-1.5 text-white/50 animate-fade-in"
              style={{ animationDelay: `${index * 0.1}s` }}
            >
              <div className="shrink-0 text-[10px] text-white/30 pt-0.5">
                {index + 1}.
              </div>
              <p className="text-[12px] leading-snug">
                {step}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
