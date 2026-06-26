import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type ReadingHistoryItem = {
  comicId: string
  albumId: string
  title: string
  author: string
  coverUrl: string
  chapterId: string
  chapterTitle: string
  chapter: string
  pageIndex: number
  pageCount: number
  updatedAt: number
}

type ReadingHistoryState = {
  items: ReadingHistoryItem[]
  upsert: (item: Omit<ReadingHistoryItem, 'updatedAt'>) => void
  remove: (comicId: string) => void
  clear: () => void
}

export const useReadingHistoryStore = create<ReadingHistoryState>()(
  persist(
    set => ({
      items: [],
      upsert: item => {
        const comicId = item.comicId.trim() || item.albumId.trim() || item.chapterId.trim()
        const nextItem: ReadingHistoryItem = {
          ...item,
          comicId,
          albumId: item.albumId.trim() || comicId,
          updatedAt: Date.now()
        }

        set(state => {
          const items = [
            nextItem,
            ...state.items.filter(entry => entry.comicId !== comicId && entry.albumId !== comicId)
          ]

          return { items }
        })
      },
      remove: comicId => {
        const targetId = comicId.trim()

        set(state => ({
          items: state.items.filter(item => item.comicId !== targetId && item.albumId !== targetId)
        }))
      },
      clear: () => {
        set({ items: [] })
      }
    }),
    {
      name: 'jm-boom-reading-history'
    }
  )
)
