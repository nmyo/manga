import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useRef } from 'react'

import { READER_GC_TIME, READER_STALE_TIME } from './constants'
import type { ReaderPageQueryKeyFactory, ReaderPageRequester } from './use-reader-page-query'

const BREEZE_ROW_PREFETCH_RADIUS = 1

export function useReaderPrefetch({
  comicId,
  endpoint,
  cacheLimitBytes,
  currentIndex,
  pageCount,
  enabled,
  pageQueryKey,
  requestPage
}: {
  comicId: string
  endpoint: string
  cacheLimitBytes: number
  currentIndex: number
  pageCount: number
  enabled: boolean
  pageQueryKey: ReaderPageQueryKeyFactory
  requestPage: ReaderPageRequester
}) {
  const queryClient = useQueryClient()
  const prefetchKeyRef = useRef('')

  useEffect(() => {
    if (!enabled) {
      return
    }

    const prefetchIndexes = readerPrefetchIndexes(
      currentIndex,
      pageCount,
      BREEZE_ROW_PREFETCH_RADIUS
    )

    if (prefetchIndexes.length === 0) {
      return
    }

    const prefetchKey = [
      endpoint,
      cacheLimitBytes,
      comicId,
      currentIndex,
      prefetchIndexes.join(',')
    ].join('|')

    if (prefetchKeyRef.current === prefetchKey) {
      return
    }

    prefetchKeyRef.current = prefetchKey
    let isActive = true

    void (async () => {
      for (const index of prefetchIndexes) {
        if (!isActive) {
          return
        }

        await queryClient
          .prefetchQuery({
            queryKey: pageQueryKey(index),
            queryFn: () => requestPage(index, 'prefetch'),
            staleTime: READER_STALE_TIME,
            gcTime: READER_GC_TIME
          })
          .catch(error => {
            if (import.meta.env.DEV) {
              console.debug('Reader page prefetch failed', error)
            }
          })
      }
    })()

    return () => {
      isActive = false
    }
  }, [
    cacheLimitBytes,
    comicId,
    currentIndex,
    enabled,
    endpoint,
    pageCount,
    pageQueryKey,
    queryClient,
    requestPage
  ])
}

function readerPrefetchIndexes(currentIndex: number, pageCount: number, radius: number) {
  const indexes: number[] = []

  for (let distance = 1; distance <= radius; distance += 1) {
    const nextIndex = currentIndex + distance
    const previousIndex = currentIndex - distance

    if (nextIndex < pageCount) {
      indexes.push(nextIndex)
    }

    if (previousIndex >= 0) {
      indexes.push(previousIndex)
    }
  }

  return indexes
}
