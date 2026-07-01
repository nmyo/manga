import { cn } from '@/lib/utils'
import type { ReaderWindowPage } from './types'

export function ReaderImageWindow({
  pages,
  currentIndex,
  pageCount,
  doublePageMode = false
}: {
  pages: ReaderWindowPage[]
  currentIndex: number
  pageCount: number
  doublePageMode?: boolean
}) {
  if (doublePageMode) {
    return <ReaderDoublePageWindow pages={pages} currentIndex={currentIndex} pageCount={pageCount} />
  }

  return (
    <div className="pointer-events-none relative h-screen w-screen overflow-hidden">
      {pages.map(page => {
        const offset = page.index - currentIndex
        const isCurrent = offset === 0

        return (
          <div
            key={page.index}
            className={cn(
              'absolute inset-0 flex h-screen w-screen items-center justify-center transition-transform duration-200 ease-out will-change-transform',
              isCurrent ? 'z-10' : 'z-0'
            )}
            style={{ transform: `translate3d(${offset * 100}%, 0, 0)` }}
          >
            <img
              src={page.src}
              alt=""
              className="h-screen w-screen select-none object-contain"
              draggable={false}
              loading="eager"
              decoding={isCurrent ? 'sync' : 'async'}
            />
          </div>
        )
      })}
    </div>
  )
}

function ReaderDoublePageWindow({
  pages,
  currentIndex,
  pageCount
}: {
  pages: ReaderWindowPage[]
  currentIndex: number
  pageCount: number
}) {
  const pageByIndex = new Map(pages.map(page => [page.index, page]))
  const leftPage = pageByIndex.get(currentIndex) ?? null
  const rightIndex = currentIndex + 1
  const rightPage = rightIndex < pageCount ? (pageByIndex.get(rightIndex) ?? null) : null
  const showRightSlot = rightIndex < pageCount

  return (
    <div className="pointer-events-none flex h-screen w-screen items-center justify-center overflow-hidden px-6 py-6">
      <div
        className={cn(
          'flex h-full w-full items-center justify-center gap-2',
          showRightSlot ? 'max-w-[1800px]' : 'max-w-[900px]'
        )}
      >
        <ReaderDoublePageSlot page={leftPage} isCurrent={true} label={`第 ${currentIndex + 1} 张`} />
        {showRightSlot ? (
          <ReaderDoublePageSlot
            page={rightPage}
            isCurrent={false}
            label={`第 ${rightIndex + 1} 张`}
          />
        ) : null}
      </div>
    </div>
  )
}

function ReaderDoublePageSlot({
  page,
  isCurrent,
  label
}: {
  page: ReaderWindowPage | null
  isCurrent: boolean
  label: string
}) {
  return (
    <div className="flex h-full min-w-0 flex-1 items-center justify-center bg-neutral-950">
      {page ? (
        <img
          src={page.src}
          alt=""
          className="max-h-full max-w-full select-none object-contain"
          draggable={false}
          loading="eager"
          decoding={isCurrent ? 'sync' : 'async'}
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center text-xs text-neutral-500">
          正在准备{label}
        </div>
      )}
    </div>
  )
}
