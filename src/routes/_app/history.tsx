import { createFileRoute } from '@tanstack/react-router'
import { Trash2Icon, XIcon } from 'lucide-react'
import { useMemo } from 'react'
import { toast } from 'sonner'

import { BackTopButton } from '@/components/back-top-button'
import { ComicCard } from '@/components/comic'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { EmptyState } from '@/components/empty-state'
import { Button } from '@/components/ui/button'
import { useHistorySelection } from '@/features/history/use-history-selection'
import { formatDate } from '@/lib/format'
import { useReadingHistoryStore } from '@/stores/reading-history-store'

export const Route = createFileRoute('/_app/history')({
  component: HistoryPage
})

function HistoryPage() {
  const items = useReadingHistoryStore(state => state.items)
  const removeMany = useReadingHistoryStore(state => state.removeMany)
  const clear = useReadingHistoryStore(state => state.clear)

  const sortedItems = useMemo(
    () => [...items].sort((left, right) => right.updatedAt - left.updatedAt),
    [items]
  )

  const selection = useHistorySelection(sortedItems)

  function deleteSelectedHistory() {
    const comicIds = [...selection.selectedComicIds]

    if (comicIds.length === 0) {
      return
    }

    removeMany(comicIds)
    selection.toggleSelectionMode(false)
    toast.success(`已删除 ${comicIds.length} 条历史观看记录`)
  }

  function clearAllHistory() {
    clear()
    toast.success('历史观看记录已清除')
  }

  return (
    <main className="relative min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-6xl space-y-6 p-4 md:p-[32px_32px_16px_96px]">
        {sortedItems.length === 0 ? (
          <EmptyState emoji="(˙ᯅ˙)" title="暂无历史观看记录" />
        ) : (
          <div className="grid grid-cols-4 gap-6">
            {sortedItems.map(item => {
              const progress = (item.pageIndex + 1) / item.pageCount
              return (
                <ComicCard
                  key={item.comicId}
                  comic={{
                    id: item.comicId,
                    title: item.title,
                    image: item.coverUrl?.trim() ?? ''
                  }}
                  ratio="square"
                  showIdBadge
                  progress={progress}
                  selectable={selection.isSelecting}
                  selected={selection.selectedComicIds.has(item.comicId)}
                  onSelect={selection.toggleSelectItem}
                  linkProps={
                    !selection.isSelecting
                      ? {
                          to: '/reader/$comicId',
                          params: { comicId: item.chapterId },
                          search: {
                            title: item.title,
                            chapter: item.chapterTitle,
                            albumId: item.albumId,
                            fromDetail: '',
                            pageIndex: String(item.pageIndex),
                            nextId: '',
                            nextChapter: ''
                          }
                        }
                      : undefined
                  }
                  metadata={
                    <>
                      <p className="line-clamp-1 text-xs text-muted-foreground">
                        {item.chapterTitle}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {item.pageIndex + 1}/{item.pageCount} • {formatDate(item.updatedAt)}
                      </p>
                    </>
                  }
                />
              )
            })}
          </div>
        )}
      </div>
      <BackTopButton />
    </div>
      <div className="shrink-0 h-14" />
    </main>
  )
}
