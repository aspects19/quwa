import { useState } from 'react'
import { Brain, ChevronDown, ChevronUp } from 'lucide-react'

interface ThinkingProcessProps {
  steps: string[]
}

export default function ThinkingProcess({ steps }: ThinkingProcessProps) {
  const [isExpanded, setIsExpanded] = useState(true)
  
  if (steps.length === 0) return null
  
  return (
    <div className="ml-4 mb-4 card border-l-4 border-accent-500/50 animate-slide-in">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="flex items-center justify-between w-full text-left group"
      >
        <div className="flex items-center gap-2">
          <div className="p-1.5 rounded-lg bg-accent-500/10">
            <Brain className="w-4 h-4 text-accent-400" />
          </div>
          <span className="text-sm font-semibold text-accent-300">
            Thinking Process
          </span>
          {steps.length > 1 && (
            <span className="text-xs text-dark-text-muted">
              ({steps.length} steps)
            </span>
          )}
        </div>
        {isExpanded ? (
          <ChevronUp className="w-4 h-4 text-dark-text-muted group-hover:text-accent-400 transition-colors" />
        ) : (
          <ChevronDown className="w-4 h-4 text-dark-text-muted group-hover:text-accent-400 transition-colors" />
        )}
      </button>
      
      {isExpanded && (
        <div className="mt-4 space-y-2">
          {steps.map((step, index) => (
            <div
              key={index}
              className="flex items-start gap-3 p-3 rounded-lg bg-dark-surface-elevated border border-white/5 animate-fade-in"
              style={{ animationDelay: `${index * 0.1}s` }}
            >
              <div className="shrink-0 w-6 h-6 rounded-full bg-accent-500/20 flex items-center justify-center text-xs font-bold text-accent-400">
                {index + 1}
              </div>
              <p className="text-sm text-dark-text-muted pt-0.5">
                {step}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
