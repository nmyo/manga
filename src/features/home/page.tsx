import { useQuery } from '@tanstack/react-query'

import { EmptyState } from '@/components/empty-state'
import { Button } from '@/components/ui/button'
import { getHomeFeed, type HomeFeedSection } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { useSettingsStore } from '@/stores/settings-store'
import { BackTopButton } from '@/components/back-top-button'
import { HomeFeedSections } from './home-feed-sections'
import { HomeFeedSkeleton } from './home-feed-skeleton'

const EMPTY_HOME_SECTIONS: HomeFeedSection[] = []

export function HomePage() {
  const endpoint = useSettingsStore(state => state.api)
  const homeFeed = useQuery({
    queryKey: queryKeys.homeFeed(endpoint),
    queryFn: () => getHomeFeed(endpoint),
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const sections = homeFeed.data?.sections ?? EMPTY_HOME_SECTIONS

  return (
    <main className="fixed inset-0 flex flex-col bg-background text-foreground">
      <div className="shrink-0 h-[env(safe-area-inset-top)]" />
      <div className="flex-1 overflow-y-auto overscroll-contain">
        <div className="p-4 md:p-[32px_80px_16px_96px]">
          <div className="min-w-0">
            {homeFeed.isLoading ? (
              <HomeFeedSkeleton />
            ) : homeFeed.isError ? (
              <EmptyState
                emoji="Ò︵Ó"
                title="数据加载失败"
                actions={
                  <Button type="button" variant="outline" size="sm" onClick={() => homeFeed.refetch()}>
                    重试
                  </Button>
                }
              />
            ) : sections.length === 0 ? (
              <EmptyState emoji="(･o･;)" title="暂无内容" />
            ) : (
              <HomeFeedSections sections={sections} />
            )}
          </div>
        </div>
        <BackTopButton />
      </div>
      <div className="shrink-0 h-14" />
    </main>
  )
}
