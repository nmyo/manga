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
  PREFETCH_SETTLE_MS,
  READER_GC_TIME,
  READER_STALE_TIME
} from './constants'
import { useSettingsStore } from '@/stores/settings-store'

export function useReaderPages(comicId: string, initialIndex = 0) {
  const endpoint = useSettingsStore(state => state.api)
  const shunt = useSettingsStore(state => state.shunt)
  const prefetchCount = useSettingsStore(state => state.prefetchCount)
  const readerCacheLimitMb = useSettingsStore(state => state.readerCacheLimitMb)
  const cacheLimitBytes = readerCacheLimitMb * 1024 * 1024
  const [currentIndex, setCurrentIndex] = useState(initialIndex)
  const [loadIndex, setLoadIndex] = useState(initialIndex)

  const manifest = useQuery({
    queryKey: ['jm-reader-manifest', endpoint, shunt, comicId],
    queryFn: () => getComicReadManifest({ readId: comicId, shunt, endpoint }),
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const pageCount = manifest.data?.pageCount ?? 0
  const page = useQuery({
    queryKey: [
      'jm-reader-page',
      endpoint,
      shunt,
      cacheLimitBytes,
      comicId,
      loadIndex,
      manifest.data?.shunt
    ],
    queryFn: () =>
      getComicReadPage({
        readId: comicId,
        index: loadIndex,
        shunt: manifest.data?.shunt ?? shunt,
        endpoint,
        cacheLimitBytes
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
  }, [comicId, endpoint, initialIndex, shunt])

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
    if (!manifest.data || pageCount === 0 || !isPageReady || prefetchCount === 0) {
      return
    }

    const timer = window.setTimeout(() => {
      void prefetchComicReadPages({
        readId: comicId,
        centerIndex: currentIndex,
        radius: prefetchCount,
        shunt: manifest.data.shunt,
        endpoint,
        cacheLimitBytes
      }).catch(error => {
        console.debug('Reader prefetch failed', error)
      })
    }, PREFETCH_SETTLE_MS)

    return () => window.clearTimeout(timer)
  }, [
    cacheLimitBytes,
    comicId,
    currentIndex,
    endpoint,
    isPageReady,
    manifest.data,
    pageCount,
    prefetchCount
  ])

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
