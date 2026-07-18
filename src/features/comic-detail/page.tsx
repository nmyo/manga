import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { useEffect, useMemo, useState } from 'react'

import { BackTopButton } from '@/components/back-top-button'
import { PageBackButton } from '@/components/page-back-button'
import {
  getComicComments,
  getComicDetail,
  toggleComicFavorite,
  type ComicDetail,
  type ComicDetailResult
} from '@/lib/api/comic'
import {
  resolveComicAlbumId,
  resolveComicStartReadingTarget,
} from '@/lib/comic'
import { getComicReadManifest } from '@/lib/api/reader'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { ChaptersSection } from './chapters'
import { CommentsDrawer } from './comments'
import { ComicHero } from './hero'
import { RelatedPanel } from './related'
import { ComicDetailSkeleton } from './shared'
import { EmptyState } from '@/components/empty-state'
import { Button } from '@/components/ui/button'
import { useSettingsStore } from '@/stores/settings-store'

export function ComicDetailPage({ comicId }: { comicId: string }) {
  const endpoint = useSettingsStore(state => state.api)
  const detail = useQuery({
    queryKey: queryKeys.comicDetail(endpoint, comicId),
    queryFn: () => getComicDetail(comicId, endpoint),
    staleTime: CACHE.DETAIL_STALE_TIME,
    gcTime: CACHE.DETAIL_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })

  return (
    <main className="min-h-screen bg-background p-4 md:p-[32px_32px_16px_96px] text-foreground">
      <div className="mx-auto max-w-7xl space-y-8">
        <PageBackButton />

        {detail.isLoading ? (
          <ComicDetailSkeleton />
        ) : detail.isError ? (
          <EmptyState
            emoji="Ò︵Ó"
            title="数据加载失败"
            actions={
              <Button type="button" variant="outline" size="sm" onClick={() => detail.refetch()}>
                重试
              </Button>
            }
          />
        ) : detail.data == null ? (
          <EmptyState emoji="(･o･;)" title="暂无详情" />
        ) : (
          <ComicDetailView comic={detail.data.comic} />
        )}
      </div>
      <BackTopButton />
    </main>
  )
}

function ComicDetailView({ comic }: { comic: ComicDetail }) {
  const endpoint = useSettingsStore(state => state.api)
  const queryClient = useQueryClient()
  const [isCommentsOpen, setIsCommentsOpen] = useState(false)
  const albumId = resolveComicAlbumId(comic)
  const startReadingTarget = useMemo(() => resolveComicStartReadingTarget(comic), [comic])

  useEffect(() => {
    const readId = startReadingTarget.readId.trim()

    if (readId.length === 0) {
      return
    }

    let isActive = true

    void queryClient
      .prefetchQuery({
        queryKey: queryKeys.readerManifest(endpoint, readId),
        queryFn: () => getComicReadManifest({ readId, endpoint }),
        staleTime: CACHE.READER_STALE_TIME,
        gcTime: CACHE.READER_GC_TIME
      })
      .catch(error => {
        if (isActive && import.meta.env.DEV) {
          console.debug('Comic detail reader manifest prefetch failed', error)
        }
      })

    return () => {
      isActive = false
    }
  }, [endpoint, queryClient, startReadingTarget.readId])

  const favoriteMutation = useMutation({
    mutationFn: async () =>
      toggleComicFavorite({
        comicId: comic.id,
        currentFavorite: comic.isFavorite,
        endpoint
      }),
    onSuccess: result => {
      queryClient.setQueryData<ComicDetailResult | undefined>(
        queryKeys.comicDetail(endpoint, comic.id),
        current => {
          if (current == null) {
            return current
          }

          return {
            ...current,
            comic: {
              ...current.comic,
              isFavorite: result.favorited
            }
          }
        }
      )
      toast.success(result.favorited ? '已添加收藏' : '已取消收藏')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '收藏操作失败')
    }
  })

  const commentsQuery = useInfiniteQuery({
    queryKey: queryKeys.comicComments(endpoint, comic.id),
    queryFn: ({ pageParam }) => getComicComments({ comicId: comic.id, page: pageParam, endpoint }),
    initialPageParam: 1,
    enabled: isCommentsOpen,
    staleTime: CACHE.COMMENTS_STALE_TIME,
    gcTime: CACHE.COMMENTS_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    getNextPageParam: (lastPage, allPages) => {
      const loadedCount = allPages.reduce((sum, page) => sum + page.comments.length, 0)

      if (lastPage.comments.length === 0 || loadedCount >= lastPage.total) {
        return undefined
      }

      return lastPage.page + 1
    }
  })
  const comments = useMemo(
    () => commentsQuery.data?.pages.flatMap(page => page.comments) ?? [],
    [commentsQuery.data]
  )
  const commentTotal = commentsQuery.data?.pages[0]?.total ?? comic.commentTotal

  return (
    <div className="space-y-10">
      <ComicHero
        comic={comic}
        onCommentsClick={() => setIsCommentsOpen(true)}
        onFavoriteClick={() => favoriteMutation.mutate()}
        favoriteBusy={favoriteMutation.isPending}
      />

      <div className="space-y-8">
        <div className="min-w-0">
          <ChaptersSection
            albumId={albumId}
            comicId={comic.id}
            comicTitle={comic.title}
            chapters={comic.series}
          />
        </div>

        <aside>
          <RelatedPanel items={comic.relatedList} />
        </aside>
      </div>

      <CommentsDrawer
        open={isCommentsOpen}
        onOpenChange={setIsCommentsOpen}
        state={{
          isLoading: commentsQuery.isLoading,
          isFetchingNextPage: commentsQuery.isFetchingNextPage,
          isError: commentsQuery.isError,
          errorMessage: commentsQuery.error?.message,
          total: commentTotal,
          comments,
          hasNextPage: commentsQuery.hasNextPage,
          onRetry: () => commentsQuery.refetch(),
          onLoadMore: () => commentsQuery.fetchNextPage({ cancelRefetch: false })
        }}
      />
    </div>
  )
}
