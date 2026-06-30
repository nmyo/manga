import { cn } from '@/lib/utils'
import type { ReaderWindowPage } from './types'

export function ReaderImageWindow({
  pages,
  currentIndex
}: {
  pages: ReaderWindowPage[]
  currentIndex: number
}) {
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
