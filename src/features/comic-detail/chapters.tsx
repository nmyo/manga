import { Link } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'

import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious
} from '@/components/ui/pagination'
import type { ComicChapter } from '@/lib/api/comic'
import {
  SINGLE_CHAPTER_TITLE,
  formatComicChapterTitle,
  getComicDisplayChapterCount,
  sortComicChapters
} from '@/lib/comic'
import { UI } from '@/lib/constants'
import { cn } from '@/lib/utils'
import { SectionHeading } from './shared'

export function ChaptersSection({
  albumId,
  comicId,
  comicTitle,
  chapters
}: {
  albumId: string
  comicId: string
  comicTitle: string
  chapters: ComicChapter[]
}) {
  const sortedChapters = useMemo(() => sortComicChapters(chapters), [chapters])
  const displayChapterCount = getComicDisplayChapterCount(chapters)
  const [page, setPage] = useState(1)
  const pageCount = Math.max(1, Math.ceil(sortedChapters.length / UI.CHAPTER_PAGE_SIZE))
  const safePage = Math.min(page, pageCount)
  const visibleChapters = sortedChapters.slice(
    (safePage - 1) * UI.CHAPTER_PAGE_SIZE,
    safePage * UI.CHAPTER_PAGE_SIZE
  )

  useEffect(() => {
    setPage(current => Math.min(current, pageCount))
  }, [pageCount])

  function changePage(nextPage: number) {
    const clampedPage = Math.min(Math.max(nextPage, 1), pageCount)
    setPage(clampedPage)
    document.getElementById('chapters')?.scrollIntoView({
      behavior: 'smooth',
      block: 'start'
    })
  }

  return (
    <section id="chapters" className="scroll-mt-8 space-y-4">
      <SectionHeading title="章节" description={`${displayChapterCount} 个章节`} />
      {sortedChapters.length === 0 ? (
        <Link
          to="/reader/$comicId"
          params={{ comicId }}
          search={{
            title: comicTitle,
            chapter: SINGLE_CHAPTER_TITLE,
            albumId,
            fromDetail: '1',
            pageIndex: '0',
            nextId: '',
            nextChapter: ''
          }}
          className="inline-flex items-center rounded-md border border-border bg-card px-4 py-2 text-sm font-medium transition-colors hover:bg-muted/60"
        >
          {SINGLE_CHAPTER_TITLE}
        </Link>
      ) : (
        <>
          <div className="flex flex-wrap gap-2">
            {visibleChapters.map((chapter, index) => {
              const chapterIndex = (safePage - 1) * UI.CHAPTER_PAGE_SIZE + index
              const nextChapter = sortedChapters[chapterIndex - 1] ?? null
              const chapterTitle = formatComicChapterTitle(chapter, chapterIndex)
              const nextChapterTitle = nextChapter
                ? formatComicChapterTitle(nextChapter, chapterIndex - 1)
                : ''

              return (
                <Link
                  key={chapter.id}
                  to="/reader/$comicId"
                  params={{ comicId: chapter.id }}
                  search={{
                    title: comicTitle,
                    chapter: chapterTitle,
                    albumId,
                    fromDetail: '1',
                    pageIndex: '0',
                    nextId: nextChapter?.id ?? '',
                    nextChapter: nextChapterTitle
                  }}
                  className="inline-flex items-center justify-center rounded-md border border-border bg-card px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted/60 min-w-[72px] truncate"
                >
                  {chapterTitle}
                </Link>
              )
            })}
          </div>

          {pageCount > 1 ? (
            <ChapterPagination page={safePage} pageCount={pageCount} onPageChange={changePage} />
          ) : null}
        </>
      )}
    </section>
  )
}

function getVisiblePages(currentPage: number, pageCount: number) {
  if (pageCount <= 7) {
    return Array.from({ length: pageCount }, (_, index) => index + 1)
  }

  const pages = new Set([1, pageCount, currentPage - 1, currentPage, currentPage + 1])
  const sortedPages = [...pages]
    .filter(page => page >= 1 && page <= pageCount)
    .sort((left, right) => left - right)
  const visiblePages: Array<number | 'ellipsis'> = []

  for (const page of sortedPages) {
    const previousPage = visiblePages[visiblePages.length - 1]

    if (typeof previousPage === 'number' && page - previousPage > 1) {
      visiblePages.push('ellipsis')
    }

    visiblePages.push(page)
  }

  return visiblePages
}

function ChapterPagination({
  page,
  pageCount,
  onPageChange
}: {
  page: number
  pageCount: number
  onPageChange: (page: number) => void
}) {
  const pages = getVisiblePages(page, pageCount)

  return (
    <Pagination className="pt-2">
      <PaginationContent>
        <PaginationItem>
          <PaginationPrevious
            href="#"
            text="上一页"
            className={cn(page === 1 && 'pointer-events-none opacity-50')}
            onClick={event => {
              event.preventDefault()
              onPageChange(page - 1)
            }}
          />
        </PaginationItem>
        {pages.map((item, index) =>
          item === 'ellipsis' ? (
            <PaginationItem key={`ellipsis-${index}`}>
              <PaginationEllipsis />
            </PaginationItem>
          ) : (
            <PaginationItem key={item}>
              <PaginationLink
                href="#"
                isActive={item === page}
                onClick={event => {
                  event.preventDefault()
                  onPageChange(item)
                }}
              >
                {item}
              </PaginationLink>
            </PaginationItem>
          )
        )}
        <PaginationItem>
          <PaginationNext
            href="#"
            text="下一页"
            className={cn(page === pageCount && 'pointer-events-none opacity-50')}
            onClick={event => {
              event.preventDefault()
              onPageChange(page + 1)
            }}
          />
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  )
}
