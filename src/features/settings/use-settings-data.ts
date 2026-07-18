import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import {
  checkAppUpdate,
  configureNetworkProxy,
  discoverApiEndpoints,
  getCurrentAppVersion,
  installAppUpdate
} from '@/lib/api/setting'
import { clearReaderCache, getReaderCacheStats } from '@/lib/api/reader'
import { getSavedLoginConfig, saveLoginCredentials, setLoginAutoLogin } from '@/lib/api/user'
import { queryKeys } from '@/lib/query-keys'

import { type ProxyMode } from '@/stores/settings-store'

export function useSettingsData(
  api: string,
  cacheLimitBytes: number,
  proxyMode: ProxyMode,
  proxyHost: string,
  proxyPort: number
) {
  const queryClient = useQueryClient()

  const endpointDiscovery = useQuery({
    queryKey: queryKeys.apiEndpointDiscovery(),
    queryFn: discoverApiEndpoints,
    staleTime: 60 * 1000,
    gcTime: 5 * 60 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })

  const readerCacheStats = useQuery({
    queryKey: queryKeys.readerCacheStats(cacheLimitBytes),
    queryFn: () => getReaderCacheStats(cacheLimitBytes),
    staleTime: 0,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false
  })

  const clearCache = useMutation({
    mutationFn: () => clearReaderCache(cacheLimitBytes),
    onSuccess: data => {
      toast.success('阅读缓存已清理')
      queryClient.setQueryData(queryKeys.readerCacheStats(cacheLimitBytes), data)
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  const savedLoginConfig = useQuery({
    queryKey: queryKeys.savedLoginConfig(),
    queryFn: getSavedLoginConfig,
    staleTime: 0,
    refetchOnWindowFocus: false
  })

  const saveAccount = useMutation({
    mutationFn: ({
      username,
      password,
      autoLogin
    }: {
      username: string
      password: string
      autoLogin: boolean
    }) => saveLoginCredentials({ username, password, endpoint: api, autoLogin }),
    onSuccess: data => {
      queryClient.setQueryData(queryKeys.savedLoginConfig(), data)
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  const setAccountAutoLogin = useMutation({
    mutationFn: setLoginAutoLogin,
    onSuccess: data => {
      if (data) {
        queryClient.setQueryData(queryKeys.savedLoginConfig(), data)
      }
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  const appVersion = useQuery({
    queryKey: queryKeys.appVersion(),
    queryFn: getCurrentAppVersion,
    staleTime: Infinity,
    gcTime: Infinity,
    retry: false,
    refetchOnWindowFocus: false
  })

  const appUpdate = useQuery({
    queryKey: queryKeys.appUpdate(),
    queryFn: () => checkAppUpdate(),
    enabled: false,
    staleTime: 24 * 60 * 60 * 1000,
    gcTime: 24 * 60 * 60 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })

  const checkUpdate = useMutation({
    mutationFn: async () => {
      await configureNetworkProxy({ mode: proxyMode, host: proxyHost, port: proxyPort })
      return checkAppUpdate({ force: true })
    },
    onSuccess: data => {
      queryClient.setQueryData(queryKeys.appUpdate(), data)

      if (data.currentVersion) {
        queryClient.setQueryData(queryKeys.appVersion(), data.currentVersion)
      }

      if (data.available) {
        toast.success(`发现新版本 ${data.version}`)
        return
      }

      toast.success('当前已是最新版本')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  const installUpdate = useMutation({
    mutationFn: async () => {
      await configureNetworkProxy({ mode: proxyMode, host: proxyHost, port: proxyPort })
      return installAppUpdate()
    },
    onSuccess: installed => {
      if (!installed) {
        toast.success('当前已是最新版本')
      }
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  return {
    endpointDiscovery,
    readerCacheStats,
    clearCache,
    savedLoginConfig,
    saveAccount,
    setAccountAutoLogin,
    appVersion,
    appUpdate,
    checkUpdate,
    installUpdate
  }
}
