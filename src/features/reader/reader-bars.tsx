import { ArrowLeftIcon, LoaderCircleIcon, RotateCwIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { ReaderChapterControls } from './reader-chapter-controls'
import { ReaderProgressSlider } from './reader-progress-slider'
import type { ReaderChapterItem } from './types'

export function ReaderTopBar({
  fallbackReadId,
  title,
  chapter,
  isFetching,
  visible,
  onBack,
  onRetry
}: {
  fallbackReadId: string
  title: string
  chapter: string
  isFetching: boolean
  visible: boolean
  onBack: () => void
  onRetry: () => void
}) {
  const displayTitle = title || `JM ${fallbackReadId}`

  return (
    <header
      className={cn(
        'absolute inset-x-0 top-0 z-30 grid h-14 md:h-16 grid-cols-[80px_minmax(0,1fr)_60px] md:grid-cols-[120px_minmax(0,1fr)_120px] items-center bg-neutral-950/85 px-2 md:px-4 backdrop-blur transition-all duration-200',
        visible ? 'translate-y-0 opacity-100' : 'pointer-events-none -translate-y-3 opacity-0'
      )}
      onClick={event => event.stopPropagation()}
    >
      <Button
        variant="ghost"
        size="sm"
        className="justify-self-start text-neutral-50 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50"
        onClick={onBack}
      >
        <ArrowLeftIcon className="size-4" />
        <span className="hidden md:inline">返回</span>
      </Button>

      <div className="min-w-0 text-center px-2">
        <div className="truncate text-xs md:text-sm font-medium text-neutral-50">{displayTitle}</div>
        {chapter ? <div className="mt-0.5 truncate text-[10px] md:text-xs text-neutral-400">{chapter}</div> : null}
      </div>

      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="重新加载"
        className="justify-self-end text-neutral-50 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50"
        onClick={onRetry}
      >
        {isFetching ? (
          <LoaderCircleIcon className="size-4 animate-spin" />
        ) : (
          <RotateCwIcon className="size-4" />
        )}
      </Button>
    </header>
  )
}

export function ReaderBottomBar({
  title,
  currentReadId,
  previousChapter,
  nextChapter,
  chapters,
  albumId,
  currentIndex,
  pageCount,
  doublePageMode,
  visible,
  onPageChange
}: {
  title: string
  currentReadId: string
  previousChapter: ReaderChapterItem | null
  nextChapter: ReaderChapterItem | null
  chapters: ReaderChapterItem[]
  albumId: string
  currentIndex: number
  pageCount: number
  doublePageMode: boolean
  visible: boolean
  onPageChange: (index: number) => void
}) {
  return (
    <footer
      className={cn(
        'absolute bottom-4 md:bottom-8 left-1/2 z-30 flex w-[calc(100vw-24px)] md:w-[480px] max-w-[480px] -translate-x-1/2 flex-col gap-2 rounded-xl border border-white/10 bg-neutral-950/85 p-2.5 md:p-3 text-neutral-50 shadow-lg backdrop-blur transition-all duration-200',
        visible ? 'translate-y-0 opacity-100' : 'pointer-events-none translate-y-3 opacity-0'
      )}
      onClick={event => event.stopPropagation()}
    >
      <ReaderProgressSlider
        currentIndex={currentIndex}
        pageCount={pageCount}
        onPageChange={onPageChange}
      />
      <ReaderChapterControls
        title={title}
        albumId={albumId}
        currentReadId={currentReadId}
        chapters={chapters}
        previousChapter={previousChapter}
        nextChapter={nextChapter}
        currentIndex={currentIndex}
        pageCount={pageCount}
        doublePageMode={doublePageMode}
      />
    </footer>
  )
}
