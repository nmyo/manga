import { useCallback, useEffect, useRef, useState } from 'react'

export function useReaderToolbarVisibility() {
  const [isVisible, setIsVisible] = useState(true)
  const timerRef = useRef<number | null>(null)

  const clearHideTimer = useCallback(() => {
    if (timerRef.current == null) {
      return
    }

    window.clearTimeout(timerRef.current)
    timerRef.current = null
  }, [])

  const hide = useCallback(() => {
    clearHideTimer()
    setIsVisible(false)
  }, [clearHideTimer])

  const show = useCallback(() => {
    clearHideTimer()
    setIsVisible(true)
    timerRef.current = window.setTimeout(() => {
      setIsVisible(false)
      timerRef.current = null
    }, 1800)
  }, [clearHideTimer])

  useEffect(() => {
    show()

    return clearHideTimer
  }, [clearHideTimer, show])

  return { isVisible, show, hide }
}
