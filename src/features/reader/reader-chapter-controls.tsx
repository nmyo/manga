import { Link } from '@tanstack/react-router'
import { ChevronLeftIcon, ChevronRightIcon, ListIcon } from 'lucide-react'
import { useState, type ReactNode } from 'react'

import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { ReaderChapterDrawer } from './reader-chapter-drawer'
import { toReaderChapterSearch } from './reader-chapter-link'
import { ReaderSettingsMenu } from './reader-settings-menu'
import type { ReaderChapterItem } from './types'

const CHAPTER_BUTTON_CLASS =
  'h-7 rounded-md px-1.5 md:px-2 text-[10px] md:text-xs text-neutral-200 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50 disabled:text-neutral-500 shrink-0'

export function ReaderChapterControls({
  title,
  albumId,
  currentReadId,
  chapters,
  previousChapter,
  nextChapter,
  currentIndex,
  pageCount,
  doublePageMode
}: {
  title: string
  albumId: string
  currentReadId: string
  chapters: ReaderChapterItem[]
  previousChapter: ReaderChapterItem | null
  nextChapter: ReaderChapterItem | null
  currentIndex: number
  pageCount: number
  doublePageMode: boolean
}) {
  const [chapterDrawerOpen, setChapterDrawerOpen] = useState(false)
  const hasChapterList = chapters.length > 1
  const hasChapterNavigation = hasChapterList || previousChapter != null || nextChapter != null
  const pageLabel =
    doublePageMode && currentIndex + 1 < pageCount
      ? `${currentIndex + 1}-${currentIndex + 2}/${pageCount}`
      : `${currentIndex + 1}/${pageCount}`

  return (
    <>
      <div className="flex w-full items-center justify-between gap-1 md:gap-3">
        <div className="flex min-w-0 flex-1 items-center gap-0.5 md:gap-1 overflow-x-auto scrollbar-none">
          <ChapterNavButton
            title={title}
            albumId={albumId}
            chapter={hasChapterNavigation ? previousChapter : null}
            chapters={chapters}
          >
            <ChevronLeftIcon className="size-3 md:size-3.5" />
            <span className="hidden sm:inline">上一章</span>
          </ChapterNavButton>

          <ChapterNavButton
            title={title}
            albumId={albumId}
            chapter={hasChapterNavigation ? nextChapter : null}
            chapters={chapters}
          >
            <span className="hidden sm:inline">下一章</span>
            <ChevronRightIcon className="size-3 md:size-3.5" />
          </ChapterNavButton>

          <Button
            type="button"
            variant="ghost"
            size="xs"
            disabled={!hasChapterList}
            className={CHAPTER_BUTTON_CLASS}
            onClick={() => setChapterDrawerOpen(true)}
          >
            <ListIcon className="size-3 md:size-3.5" />
            <span className="hidden sm:inline">目录</span>
          </Button>

          <ReaderSettingsMenu />
        </div>

        <div className="shrink-0 text-[10px] md:text-xs text-neutral-300 tabular-nums min-w-[48px] text-right">
          {pageLabel}
        </div>
      </div>

      <ReaderChapterDrawer
        open={chapterDrawerOpen}
        onOpenChange={setChapterDrawerOpen}
        title={title}
        albumId={albumId}
        currentReadId={currentReadId}
        chapters={chapters}
      />
    </>
  )
}

function ChapterNavButton({
  title,
  albumId,
  chapter,
  chapters,
  children
}: {
  title: string
  albumId: string
  chapter: ReaderChapterItem | null
  chapters: ReaderChapterItem[]
  children: ReactNode
}) {
  if (!chapter) {
    return (
      <Button variant="ghost" size="xs" disabled className={CHAPTER_BUTTON_CLASS}>
        {children}
      </Button>
    )
  }

  return (
    <Button asChild variant="ghost" size="xs" className={cn(CHAPTER_BUTTON_CLASS)}>
      <Link
        to="/reader/$comicId"
        params={{ comicId: chapter.id }}
        replace
        search={toReaderChapterSearch({ title, albumId, chapter, chapters })}
      >
        {children}
      </Link>
    </Button>
  )
}
