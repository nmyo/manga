import { Link } from '@tanstack/react-router'

import { ComicCover } from '@/components/comic-cover'
import { Card, CardContent } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import { type ReadingHistoryItem } from '@/stores/reading-history-store'

interface HistoryCardProps {
  item: ReadingHistoryItem
  isSelecting: boolean
  isSelected: boolean
  onSelectionChange: (comicId: string, checked: boolean) => void
}

export function HistoryCard({
  item,
  isSelecting,
  isSelected,
  onSelectionChange
}: HistoryCardProps) {
  const coverSrc = item.coverUrl?.trim() ?? ''
  const progress = item.pageCount > 0 ? ((item.pageIndex + 1) / item.pageCount) * 100 : 0
  const title = item.title || `JM ${item.comicId}`
  const card = (
    <Card
      size="sm"
      role={isSelecting ? 'button' : undefined}
      aria-pressed={isSelecting ? isSelected : undefined}
      tabIndex={isSelecting ? 0 : undefined}
      className={cn(
        'gap-0 overflow-hidden py-0 transition-shadow hover:cursor-pointer hover:shadow-xl'
      )}
      onClick={isSelecting ? () => onSelectionChange(item.comicId, !isSelected) : undefined}
      onKeyDown={
        isSelecting
          ? event => {
              if (event.key !== 'Enter' && event.key !== ' ') {
                return
              }

              event.preventDefault()
              onSelectionChange(item.comicId, !isSelected)
            }
          : undefined
      }
    >
      <div className="relative">
        {isSelecting ? (
          <div className="absolute top-4 right-4 z-30">
            <Checkbox
              checked={isSelected}
              className="data-checked:border-green-500 data-checked:bg-green-500 dark:data-checked:border-green-500 dark:data-checked:bg-green-500"
              onClick={event => event.stopPropagation()}
              onKeyDown={event => event.stopPropagation()}
              onCheckedChange={checked => onSelectionChange(item.comicId, checked === true)}
            />
          </div>
        ) : null}
        <ComicCover id={item.comicId} title={title} image={coverSrc} ratio="square" showIdBadge />
        <div className="absolute right-2 bottom-2 left-2 z-20">
          <div className="h-1 overflow-hidden rounded-full bg-black/40">
            <div className="h-full rounded-full bg-primary" style={{ width: `${progress}%` }} />
          </div>
        </div>
      </div>
      <CardContent className="space-y-1.5 p-3">
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="truncate text-sm font-semibold">{title}</div>
          </TooltipTrigger>
          <TooltipContent side="top">{title}</TooltipContent>
        </Tooltip>
        <p className="line-clamp-1 text-xs text-muted-foreground">{item.chapterTitle}</p>
        {item.author ? (
          <p className="line-clamp-1 text-xs text-muted-foreground">{item.author}</p>
        ) : null}
        <p className="text-xs text-muted-foreground">
          {item.pageIndex + 1}/{item.pageCount} • {formatDate(item.updatedAt)}
        </p>
      </CardContent>
    </Card>
  )

  if (isSelecting) {
    return <div className="block">{card}</div>
  }

  return (
    <Link
      to="/reader/$comicId"
      params={{ comicId: item.chapterId }}
      search={{
        title,
        chapter: item.chapterTitle,
        albumId: item.albumId,
        fromDetail: '',
        pageIndex: String(item.pageIndex),
        nextId: '',
        nextChapter: ''
      }}
      className="block"
    >
      {card}
    </Link>
  )
}

function formatDate(value: number) {
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  }).format(new Date(value))
}
