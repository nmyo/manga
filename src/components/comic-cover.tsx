import { ImageIcon } from 'lucide-react'
import { memo, useState } from 'react'

import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings-store'

export type ComicCoverRatio = 'portrait' | 'square'

export type ComicCoverProps = {
  id?: string
  title: string
  image: string
  className?: string
  ratio?: ComicCoverRatio
  showIdBadge?: boolean
}

const COVER_RATIO_CLASS: Record<ComicCoverRatio, string> = {
  portrait: 'aspect-3/4',
  square: 'aspect-square'
}

export const ComicCover = memo(function ComicCover({
  id,
  image,
  className,
  ratio = 'portrait',
  showIdBadge = false
}: ComicCoverProps) {
  const [failedImage, setFailedImage] = useState('')
  const hideCovers = useSettingsStore(state => state.hideCovers)

  const hasImage = image.length > 0
  const hasImageError = failedImage === image
  const shouldShowImage = hasImage && !hasImageError && !hideCovers

  return (
    <div className={cn('relative overflow-hidden bg-muted', COVER_RATIO_CLASS[ratio], className)}>
      {shouldShowImage ? (
        <img
          src={image}
          loading="lazy"
          decoding="async"
          referrerPolicy="no-referrer"
          className="h-full w-full object-cover"
          onError={() => setFailedImage(image)}
        />
      ) : (
        <CoverPlaceholder />
      )}

      {showIdBadge && id ? (
        <div className="absolute top-2 left-2 rounded-full border bg-background/45 px-2 py-1 text-[10px] backdrop-blur">
          JM {id}
        </div>
      ) : null}
    </div>
  )
})

function CoverPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center text-muted-foreground">
      <ImageIcon className="size-6" />
    </div>
  )
}
