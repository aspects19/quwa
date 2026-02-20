import { useState, useEffect, useRef } from 'react'
import { Loader2 } from 'lucide-react'

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
    <div
      className="flex items-center gap-2 text-white/50 animate-pulse"
      style={{ opacity, transition: 'opacity 0.3s ease' }}
    >
      <Loader2 className="w-4 h-4 animate-spin shrink-0" />
      <span className="text-sm">{currentStep}</span>
    </div>
  )
}
