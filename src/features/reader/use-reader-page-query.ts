import { useQuery } from '@tanstack/react-query'
import { useCallback, useMemo } from 'react'

import {
  type ComicReadPageResult,
  getComicReadPage,
  readerFileSrc
} from '@/lib/api/reader'
import { queryKeys } from '@/lib/query-keys'
import { READER_GC_TIME, READER_STALE_TIME } from './constants'

export type ReaderPageRequestOrigin = 'visible' | 'prefetch'
export type ReaderPageQueryKeyFactory = (index: number) => ReturnType<typeof queryKeys.readerPage>
export type ReaderPageRequester = (
  index: number,
  requestOrigin: ReaderPageRequestOrigin
) => Promise<ComicReadPageResult>

export function useReaderPageQuery({
  comicId,
  endpoint,
  cacheLimitBytes,
  pageIndex,
  enabled
}: {
  comicId: string
  endpoint: string
  cacheLimitBytes: number
  pageIndex: number
  enabled: boolean
}) {
  const pageQueryKey = useCallback<ReaderPageQueryKeyFactory>(
    (index: number) => queryKeys.readerPage(endpoint, comicId, cacheLimitBytes, index),
    [cacheLimitBytes, comicId, endpoint]
  )
  const requestPage = useCallback<ReaderPageRequester>(
    (index: number, requestOrigin: ReaderPageRequestOrigin) =>
      getComicReadPage({
        readId: comicId,
        index,
        endpoint,
        requestOrigin,
        cacheLimitBytes
      }),
    [cacheLimitBytes, comicId, endpoint]
  )
  const page = useQuery({
    queryKey: pageQueryKey(pageIndex),
    queryFn: () => requestPage(pageIndex, 'visible'),
    enabled,
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const isPageReady = page.data?.index === pageIndex
  const pageSrc = useMemo(
    () => (isPageReady && page.data ? readerFileSrc(page.data.path) : ''),
    [isPageReady, page.data]
  )

  return {
    page,
    pageSrc,
    isPageReady,
    pageQueryKey,
    requestPage
  }
}
