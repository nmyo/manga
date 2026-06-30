import { useCallback, useEffect, useState } from 'react'

export function useReaderNavigation({
  comicId,
  endpoint,
  initialIndex,
  pageCount
}: {
  comicId: string
  endpoint: string
  initialIndex: number
  pageCount: number
}) {
  const initialPageIndex = normalizePageIndex(initialIndex)
  const [currentIndex, setCurrentIndex] = useState(initialPageIndex)
  const clampPageIndex = useCallback(
    (index: number) => Math.min(Math.max(index, 0), Math.max(pageCount - 1, 0)),
    [pageCount]
  )
  const effectiveCurrentIndex = pageCount > 0 ? clampPageIndex(currentIndex) : currentIndex
  const goToPreviousPage = useCallback(() => {
    if (pageCount === 0) {
      return
    }

    setCurrentIndex(index => clampPageIndex(index - 1))
  }, [clampPageIndex, pageCount])
  const goToNextPage = useCallback(() => {
    if (pageCount === 0) {
      return
    }

    setCurrentIndex(index => clampPageIndex(index + 1))
  }, [clampPageIndex, pageCount])

  useEffect(() => {
    setCurrentIndex(initialPageIndex)
  }, [comicId, endpoint, initialPageIndex])

  useEffect(() => {
    if (currentIndex < pageCount || pageCount === 0) {
      return
    }

    setCurrentIndex(Math.max(0, pageCount - 1))
  }, [currentIndex, pageCount])

  return {
    currentIndex,
    effectiveCurrentIndex,
    isLastPage: pageCount > 0 && currentIndex >= pageCount - 1,
    goToPreviousPage,
    goToNextPage
  }
}

function normalizePageIndex(index: number) {
  if (!Number.isFinite(index)) {
    return 0
  }

  return Math.max(0, Math.floor(index))
}
