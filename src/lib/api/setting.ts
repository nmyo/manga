import { getVersion } from '@tauri-apps/api/app'
import { hasTauriRuntime, tauriInvoke } from './tauri'

export type RemoteSettingParams = {
  endpoint?: string | null
}

export type RemoteSetting = {
  endpoint: string
  imgHost: string
}

export type ApiEndpointProbe = {
  endpoint: string
  available: boolean
  latencyMs: number | null
  imgHost: string | null
  error: string | null
}

export type NetworkProxyMode = 'off' | 'http' | 'socks5'

export type AppUpdateCheckResult = {
  currentVersion: string
  available: boolean
  version: string | null
  notes: string | null
  pubDate: string | null
}

export type DiagnosticsInfo = {
  logDir: string
  debugLoggingEnabled: boolean
}

const GITHUB_REPO = 'nmyo/manga'

export async function getRemoteSetting({
  endpoint = null
}: RemoteSettingParams = {}): Promise<RemoteSetting> {
  return tauriInvoke<RemoteSetting>(
    'get_remote_setting',
    { endpoint },
    'Remote setting needs the Tauri desktop runtime.'
  )
}

export async function discoverApiEndpoints(): Promise<ApiEndpointProbe[]> {
  return tauriInvoke<ApiEndpointProbe[]>(
    'discover_api_endpoints',
    undefined,
    'API endpoint discovery needs the Tauri desktop runtime.'
  )
}

export async function configureNetworkProxy({
  mode,
  host,
  port
}: {
  mode: NetworkProxyMode
  host: string
  port: number
}): Promise<void> {
  if (!hasTauriRuntime()) {
    return
  }

  return tauriInvoke<void>('configure_network_proxy', { mode, host, port })
}

export async function getCurrentAppVersion(): Promise<string> {
  if (!hasTauriRuntime()) {
    return ''
  }

  return getVersion()
}

export async function checkAppUpdate({
  force = false
}: {
  force?: boolean
} = {}): Promise<AppUpdateCheckResult> {
  if (!hasTauriRuntime()) {
    return {
      currentVersion: '',
      available: false,
      version: null,
      notes: null,
      pubDate: null
    }
  }

  try {
    return await tauriInvoke<AppUpdateCheckResult>('check_app_update', { force })
  } catch {
    return checkUpdateFromGitHub()
  }
}

async function checkUpdateFromGitHub(): Promise<AppUpdateCheckResult> {
  let currentVersion = ''
  try {
    currentVersion = await getVersion()
  } catch {
    // ignore
  }

  try {
    const res = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`)
    if (!res.ok) {
      return { currentVersion, available: false, version: null, notes: null, pubDate: null }
    }
    const data = await res.json()
    const latestVersion = (data.tag_name as string) ?? ''
    const current = currentVersion.replace(/^v/, '')
    const latest = latestVersion.replace(/^v/, '')
    const available = latest !== '' && latest !== current

    return {
      currentVersion,
      available,
      version: latestVersion,
      notes: data.body ?? null,
      pubDate: data.published_at ?? null
    }
  } catch {
    return { currentVersion, available: false, version: null, notes: null, pubDate: null }
  }
}

export async function installAppUpdate(): Promise<boolean> {
  if (!hasTauriRuntime()) {
    return false
  }

  try {
    return await tauriInvoke<boolean>('install_app_update')
  } catch {
    const url = `https://github.com/${GITHUB_REPO}/releases/latest`
    window.open(url, '_blank', 'noopener,noreferrer')
    return false
  }
}

export function openReleasePage(): void {
  const url = `https://github.com/${GITHUB_REPO}/releases/latest`
  if (hasTauriRuntime()) {
    import('@tauri-apps/plugin-opener').then(({ openUrl }) => {
      openUrl(url).catch(() => {
        window.open(url, '_blank', 'noopener,noreferrer')
      })
    }).catch(() => {
      window.open(url, '_blank', 'noopener,noreferrer')
    })
  } else {
    window.open(url, '_blank', 'noopener,noreferrer')
  }
}
