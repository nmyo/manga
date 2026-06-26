import { createFileRoute, Outlet, useNavigate, useRouterState } from '@tanstack/react-router'
import { CalendarDaysIcon, HeartIcon, HouseIcon, SettingsIcon, UserRoundIcon } from 'lucide-react'
import { useEffect, useState } from 'react'

import { FloatingNav, type FloatingNavItem } from '@/components/floating-nav'
import { LoginDialog } from '@/features/user/login-dialog'
import { configureNetworkProxy } from '@/lib/api/setting'
import { useSettingsStore } from '@/stores/settings-store'
import { useUserStore } from '@/stores/user-store'

export const Route = createFileRoute('/_app')({
  component: AppRoute
})

function AppRoute() {
  const navigate = useNavigate()
  const user = useUserStore(state => state.user)
  const proxyMode = useSettingsStore(state => state.proxyMode)
  const proxyHost = useSettingsStore(state => state.proxyHost)
  const proxyPort = useSettingsStore(state => state.proxyPort)
  const [isLoginOpen, setIsLoginOpen] = useState(false)
  const pathname = useRouterState({
    select: state => state.location.pathname
  })

  useEffect(() => {
    void configureNetworkProxy({
      mode: proxyMode,
      host: proxyHost,
      port: proxyPort
    }).catch(error => {
      console.error('Failed to configure network proxy', error)
    })
  }, [proxyHost, proxyMode, proxyPort])

  const items: FloatingNavItem[] = [
    { id: 'home', icon: HouseIcon, label: 'Home', to: '/' },
    { id: 'weekly', icon: CalendarDaysIcon, label: 'Weekly', to: '/weekly' },
    { id: 'favorites', icon: HeartIcon, label: 'Favorites', to: '/favorites' },
    { id: 'settings', icon: SettingsIcon, label: 'Settings', to: '/settings' },
    { id: 'me', icon: UserRoundIcon, label: 'Me', to: '/me' }
  ]

  const activeId = pathname.startsWith('/favorites')
    ? 'favorites'
    : pathname.startsWith('/settings')
      ? 'settings'
      : pathname.startsWith('/me')
        ? 'me'
        : pathname.startsWith('/weekly')
          ? 'weekly'
          : 'home'

  return (
    <div className="relative h-screen">
      <FloatingNav
        items={items}
        activeId={activeId}
        onItemClick={(item, event) => {
          if (item.id !== 'me' || user) {
            return
          }

          event.preventDefault()
          setIsLoginOpen(true)
        }}
        className="sm:top-1/2 sm:bottom-auto sm:left-6 sm:translate-x-0 sm:-translate-y-1/2"
      />
      <LoginDialog
        open={isLoginOpen}
        onOpenChange={setIsLoginOpen}
        onLoginSuccess={() => void navigate({ to: '/me' })}
      />
      <Outlet />
    </div>
  )
}
