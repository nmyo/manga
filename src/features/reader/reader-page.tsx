import { useRouter } from '@tanstack/react-router'
import { useCallback } from 'react'

import { ReaderBottomBar, ReaderTopBar } from './reader-bars'
import { ReaderHotZones } from './reader-hot-zones'
import { ReaderImage } from './reader-image'
import { ReaderError, ReaderLoading } from './reader-state'
import type { ReaderSearch } from './types'
import { useReaderChapterInfo } from './use-reader-chapter-info'
import { useReaderKeyboardNavigation } from './use-reader-keyboard-navigation'
import { useReaderPages } from './use-reader-pages'
import { useReaderToolbarVisibility } from './use-reader-toolbar-visibility'

export function ReaderPage({ comicId, search }: { comicId: string; search: ReaderSearch }) {
  const router = useRouter()
  const {
    isVisible: isToolbarVisible,
    show: showToolbar,
    hide: hideToolbar
  } = useReaderToolbarVisibility()
  const { albumId, title, chapter, nextChapter } = useReaderChapterInfo({ comicId, search })
  const {
    currentIndex,
    pageCount,
    pageSrc,
    isLastPage,
    isSettlingPage,
    isManifestLoading,
    manifestError,
    isPageLoading,
    pageError,
    isFetching,
    goToPreviousPage,
    goToNextPage,
    retry
  } = useReaderPages(comicId)
  const goBack = useCallback(() => router.history.back(), [router])

  useReaderKeyboardNavigation({
    onPrevious: goToPreviousPage,
    onNext: goToNextPage,
    onBack: goBack,
    onNavigate: hideToolbar
  })

  return (
    <main
      className="relative flex h-screen overflow-hidden bg-neutral-950 text-neutral-50"
      onMouseMove={showToolbar}
    >
      <ReaderTopBar
        fallbackReadId={comicId}
        title={title}
        chapter={chapter}
        isFetching={isFetching}
        visible={isToolbarVisible}
        onBack={goBack}
        onRetry={retry}
      />

      <ReaderHotZones onPrevious={goToPreviousPage} onNext={goToNextPage} />

      <section className="flex min-w-0 flex-1 items-center justify-center">
        {isManifestLoading ? (
          <ReaderLoading label="正在加载阅读信息" />
        ) : manifestError ? (
          <ReaderError title="阅读信息加载失败" description={manifestError.message} />
        ) : isSettlingPage || isPageLoading ? (
          <ReaderLoading label="正在准备图片" />
        ) : pageError ? (
          <ReaderError title="图片加载失败" description={pageError.message} />
        ) : pageSrc.length > 0 ? (
          <ReaderImage src={pageSrc} />
        ) : (
          <ReaderError title="暂无图片" description="当前页没有可展示的图片" />
        )}
      </section>

      <ReaderBottomBar
        title={title}
        nextChapter={nextChapter}
        albumId={albumId}
        currentIndex={currentIndex}
        pageCount={pageCount}
        showNextChapter={isLastPage && nextChapter != null}
        visible={isToolbarVisible}
      />
    </main>
  )
}
