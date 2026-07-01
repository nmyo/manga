import { useEffect } from 'react'

import type { ReaderReadMode } from '@/stores/settings-store'

export function useReaderKeyboardNavigation({
  readMode,
  onPrevious,
  onNext,
  onScrollPrevious,
  onScrollNext,
  onBack,
  onNavigate
}: {
  readMode: ReaderReadMode
  onPrevious: () => void
  onNext: () => void
  onScrollPrevious: () => void
  onScrollNext: () => void
  onBack: () => void
  onNavigate: () => void
}) {
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (readMode === 'strip' && isStripPreviousKey(event)) {
        event.preventDefault()
        onNavigate()
        onScrollPrevious()
        return
      }

      if (readMode === 'strip' && isStripNextKey(event)) {
        event.preventDefault()
        onNavigate()
        onScrollNext()
        return
      }

      if (event.key === 'ArrowLeft') {
        event.preventDefault()
        onNavigate()
        onPrevious()
        return
      }

      if (event.key === 'ArrowRight') {
        event.preventDefault()
        onNavigate()
        onNext()
        return
      }

      if (event.key === 'Escape') {
        event.preventDefault()
        onNavigate()
        onBack()
      }
    }

    window.addEventListener('keydown', handleKeyDown)

    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onBack, onNavigate, onNext, onPrevious, onScrollNext, onScrollPrevious, readMode])
}

function isStripNextKey(event: KeyboardEvent) {
  return ['ArrowDown', 'ArrowRight', 's', 'S', 'd', 'D'].includes(event.key)
}

function isStripPreviousKey(event: KeyboardEvent) {
  return ['ArrowUp', 'ArrowLeft', 'w', 'W', 'a', 'A'].includes(event.key)
}
