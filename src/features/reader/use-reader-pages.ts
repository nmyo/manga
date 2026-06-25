import { useQuery } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useState } from 'react'

import {
  getComicReadManifest,
  getComicReadPage,
  prefetchComicReadPages,
  readerFileSrc
} from '@/lib/api/reader'
import {
  PAGE_LOAD_DEBOUNCE_MS,
  PREFETCH_RADIUS,
  PREFETCH_SETTLE_MS,
  READER_GC_TIME,
  READER_STALE_TIME
} from './constants'

export function useReaderPages(comicId: string) {
  const [currentIndex, setCurrentIndex] = useState(0)
  const [loadIndex, setLoadIndex] = useState(0)

  const manifest = useQuery({
    queryKey: ['jm-reader-manifest', comicId],
    queryFn: () => getComicReadManifest({ readId: comicId }),
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const pageCount = manifest.data?.pageCount ?? 0
  const page = useQuery({
    queryKey: ['jm-reader-page', comicId, loadIndex, manifest.data?.shunt],
    queryFn: () =>
      getComicReadPage({
        readId: comicId,
        index: loadIndex,
        shunt: manifest.data?.shunt
      }),
    enabled: manifest.isSuccess && pageCount > 0,
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })

  const clampPageIndex = useCallback(
    (index: number) => Math.min(Math.max(index, 0), Math.max(pageCount - 1, 0)),
    [pageCount]
  )
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
  const retry = useCallback(() => {
    if (manifest.isError) {
      void manifest.refetch()
      return
    }

    void page.refetch()
  }, [manifest, page])
  const isSettlingPage = currentIndex !== loadIndex
  const isPageReady = !isSettlingPage && page.data?.index === currentIndex
  const pageSrc = useMemo(
    () => (isPageReady && page.data ? readerFileSrc(page.data.path) : ''),
    [isPageReady, page.data]
  )
  const isLastPage = pageCount > 0 && currentIndex >= pageCount - 1

  useEffect(() => {
    setCurrentIndex(0)
    setLoadIndex(0)
  }, [comicId])

  useEffect(() => {
    if (currentIndex < pageCount || pageCount === 0) {
      return
    }

    setCurrentIndex(Math.max(0, pageCount - 1))
  }, [currentIndex, pageCount])

  useEffect(() => {
    if (pageCount === 0) {
      return
    }

    const clampedIndex = clampPageIndex(currentIndex)

    if (clampedIndex !== currentIndex) {
      setCurrentIndex(clampedIndex)
      return
    }

    const timer = window.setTimeout(() => {
      setLoadIndex(clampedIndex)
    }, PAGE_LOAD_DEBOUNCE_MS)

    return () => window.clearTimeout(timer)
  }, [clampPageIndex, currentIndex, pageCount])

  useEffect(() => {
    if (!manifest.data || pageCount === 0 || !isPageReady) {
      return
    }

    const timer = window.setTimeout(() => {
      void prefetchComicReadPages({
        readId: comicId,
        centerIndex: currentIndex,
        radius: PREFETCH_RADIUS,
        shunt: manifest.data.shunt
      }).catch(error => {
        console.debug('Reader prefetch failed', error)
      })
    }, PREFETCH_SETTLE_MS)

    return () => window.clearTimeout(timer)
  }, [comicId, currentIndex, isPageReady, manifest.data, pageCount])

  return {
    currentIndex,
    pageCount,
    pageSrc,
    isLastPage,
    isSettlingPage,
    isManifestLoading: manifest.isLoading,
    manifestError: manifest.isError ? manifest.error : null,
    isPageLoading: page.isLoading || page.isFetching,
    pageError: page.isError ? page.error : null,
    isFetching: manifest.isFetching || page.isFetching,
    goToPreviousPage,
    goToNextPage,
    retry
  }
}
