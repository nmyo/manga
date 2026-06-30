import { useSettingsStore } from '@/stores/settings-store'
import { useReaderManifestQuery } from './use-reader-manifest-query'
import { useReaderNavigation } from './use-reader-navigation'
import { useReaderPageQuery } from './use-reader-page-query'
import { useReaderPrefetch } from './use-reader-prefetch'

export function useReaderPages(comicId: string, initialIndex = 0) {
  const endpoint = useSettingsStore(state => state.api)
  const readerCacheLimitMb = useSettingsStore(state => state.readerCacheLimitMb)
  const cacheLimitBytes = readerCacheLimitMb * 1024 * 1024
  const manifest = useReaderManifestQuery(comicId, endpoint)
  const pageCount = manifest.data?.pageCount ?? 0
  const {
    currentIndex,
    effectiveCurrentIndex,
    isLastPage,
    goToPreviousPage,
    goToNextPage
  } = useReaderNavigation({
    comicId,
    endpoint,
    initialIndex,
    pageCount
  })
  const { page, pageSrc, isPageReady, pageQueryKey, requestPage } = useReaderPageQuery({
    comicId,
    endpoint,
    cacheLimitBytes,
    pageIndex: effectiveCurrentIndex,
    enabled: manifest.isSuccess && pageCount > 0,
  })
  useReaderPrefetch({
    cacheLimitBytes,
    comicId,
    endpoint,
    currentIndex: effectiveCurrentIndex,
    pageCount,
    enabled: manifest.isSuccess && isPageReady && pageCount > 0,
    pageQueryKey,
    requestPage
  })
  const retry = () => {
    if (manifest.isError) {
      void manifest.refetch()
      return
    }

    void page.refetch()
  }

  return {
    currentIndex,
    pageCount,
    pageSrc,
    isLastPage,
    isManifestLoading: manifest.isLoading,
    manifestError: manifest.isError ? manifest.error : null,
    isPageLoading: page.isLoading && !page.data,
    pageError: page.isError ? page.error : null,
    isFetching: manifest.isFetching || page.isFetching,
    goToPreviousPage,
    goToNextPage,
    retry
  }
}
