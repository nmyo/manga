import { useQuery } from '@tanstack/react-query'

import { getComicReadManifest } from '@/lib/api/reader'
import { queryKeys } from '@/lib/query-keys'
import { READER_GC_TIME, READER_STALE_TIME } from './constants'

export function useReaderManifestQuery(comicId: string, endpoint: string) {
  return useQuery({
    queryKey: queryKeys.readerManifest(endpoint, comicId),
    queryFn: () => getComicReadManifest({ readId: comicId, endpoint }),
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
}
