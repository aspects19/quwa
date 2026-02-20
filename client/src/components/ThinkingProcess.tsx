import { useState, useEffect, useRef } from 'react'

interface ThinkingProcessProps {
  steps: string[]
}

export default function ThinkingProcess({ steps }: ThinkingProcessProps) {
  const [visibleIndex, setVisibleIndex] = useState(0)
  const [opacity, setOpacity] = useState(1)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const prevLengthRef = useRef(steps.length)

  // When new steps arrive, advance to the latest one
  useEffect(() => {
    if (steps.length === 0) return

    // A new step was pushed in
    if (steps.length > prevLengthRef.current) {
      prevLengthRef.current = steps.length

      // Clear any running timer
      if (timerRef.current) clearTimeout(timerRef.current)

      // Fade out current, then show new step
      setOpacity(0)
      timerRef.current = setTimeout(() => {
        setVisibleIndex(steps.length - 1)
        setOpacity(1)
      }, 300)
    }

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
    }
  }, [steps.length])

  if (steps.length === 0) return null

  const currentStep = steps[visibleIndex] ?? ''

  return (
    <div className="ml-10 mb-3 rounded-lg bg-white/5 px-3 py-2.5 text-xs text-white/50 backdrop-blur-sm animate-slide-in">
      {/* Header */}
      <div className="flex items-center gap-2 mb-2">
        <span className="inline-block h-2 w-2 rounded-full bg-white/40 animate-pulse" />
        <span className="text-[11px] uppercase tracking-wider text-white/40">Thinking</span>
        <span className="text-[11px] text-white/25">
          {visibleIndex + 1} / {steps.length}
        </span>
      </div>

      {/* Single step with fade transition */}
      <div
        className="flex items-start gap-2 rounded-md bg-white/5 px-2 py-1.5"
        style={{
          opacity,
          transition: 'opacity 0.3s ease',
        }}
      >
        <div className="shrink-0 text-[10px] text-white/30 pt-0.5">
          {visibleIndex + 1}.
        </div>
        <p className="text-[12px] leading-snug text-white/60">
          {currentStep}
        </p>
      </div>
    </div>
  )
}
