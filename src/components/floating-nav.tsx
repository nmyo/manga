import { Link } from '@tanstack/react-router'
import type { LucideIcon } from 'lucide-react'
import { type MouseEvent } from 'react'

import type { FileRoutesByTo } from '@/routeTree.gen'

type FloatingNavTo = keyof FileRoutesByTo

export type FloatingNavItem = {
  id: string
  icon: LucideIcon
  label: string
  to: FloatingNavTo
  separatorBefore?: boolean
}

type FloatingNavProps = {
  items: FloatingNavItem[]
  activeId: string | undefined
  onItemClick: (item: FloatingNavItem, event: MouseEvent<HTMLAnchorElement>) => void
}

export function FloatingNav({ items, activeId, onItemClick }: FloatingNavProps) {
  if (items.length === 0) return null

  const navItems = items.filter(item => !item.separatorBefore)

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 border-t border-border/70 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 safe-area-bottom">
      <ul className="flex items-stretch">
        {navItems.map(item => (
          <li key={item.id} className="flex-1">
            <Link
              to={item.to}
              onClick={(event) => onItemClick(item, event)}
              className={`flex flex-col items-center justify-center gap-0.5 py-2 transition-colors ${
                item.id === activeId
                  ? 'text-primary'
                  : 'text-muted-foreground'
              }`}
            >
              <item.icon className="size-5" />
              <span className="text-[10px] leading-tight">{item.label}</span>
            </Link>
          </li>
        ))}
      </ul>
    </nav>
  )
}
