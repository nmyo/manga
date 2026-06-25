import { useEffect } from 'react'

export function useReaderKeyboardNavigation({
  onPrevious,
  onNext,
  onBack,
  onNavigate
}: {
  onPrevious: () => void
  onNext: () => void
  onBack: () => void
  onNavigate: () => void
}) {
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'ArrowLeft') {
        event.preventDefault()
        onNavigate()
        onPrevious()
      }

      if (event.key === 'ArrowRight') {
        event.preventDefault()
        onNavigate()
        onNext()
      }

      if (event.key === 'Escape') {
        event.preventDefault()
        onNavigate()
        onBack()
      }
    }

    window.addEventListener('keydown', handleKeyDown)

    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onBack, onNavigate, onNext, onPrevious])
}
