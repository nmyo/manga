import { Link } from '@tanstack/react-router'
import type { LucideIcon } from 'lucide-react'
import { Fragment, type MouseEvent } from 'react'

import { buttonVariants } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
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

  // 移动端：底部导航栏
  // 桌面端：左侧浮动导航栏
  return (
    <>
      {/* 移动端底部导航栏 */}
      <nav className="fixed bottom-0 left-0 right-0 z-50 border-t border-border/70 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 md:hidden">
        <ul className="flex items-center justify-around px-2 py-1">
          {items.filter(item => !item.separatorBefore).slice(0, 5).map(item => (
            <NavItem 
              key={item.id} 
              item={item} 
              isActive={item.id === activeId} 
              onItemClick={onItemClick}
              mobile
            />
          ))}
        </ul>
      </nav>

      {/* 桌面端左侧浮动导航栏 */}
      <nav className="fixed top-1/2 left-6 z-50 -translate-y-1/2 rounded-full border border-border/70 p-1 backdrop-blur hidden md:block">
        <ul className="flex flex-col items-center gap-1">
          {items.map(item => (
            <Fragment key={item.id}>
              {item.separatorBefore ? (
                <li aria-hidden="true" className="my-1 h-px w-6 bg-border/70" />
              ) : null}
              <NavItem item={item} isActive={item.id === activeId} onItemClick={onItemClick} />
            </Fragment>
          ))}
        </ul>
      </nav>
    </>
  )
}

type NavItemProps = {
  item: FloatingNavItem
  isActive: boolean
  onItemClick: (item: FloatingNavItem, event: MouseEvent<HTMLAnchorElement>) => void
  mobile?: boolean
}

function NavItem({ item, isActive, onItemClick, mobile }: NavItemProps) {
  function handleClick(event: MouseEvent<HTMLAnchorElement>) {
    onItemClick(item, event)
  }

  if (mobile) {
    return (
      <li>
        <Link
          to={item.to}
          onClick={handleClick}
          className={`flex flex-col items-center gap-0.5 px-3 py-1.5 rounded-lg transition-colors ${
            isActive 
              ? 'text-primary bg-primary/10' 
              : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          <item.icon className="size-5" />
          <span className="text-[10px] font-medium">{item.label}</span>
        </Link>
      </li>
    )
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Link
          to={item.to}
          onClick={handleClick}
          className={buttonVariants({ variant: isActive ? 'default' : 'ghost', size: 'icon' })}
        >
          <item.icon className="size-4" />
        </Link>
      </TooltipTrigger>
      <TooltipContent side="right">{item.label}</TooltipContent>
    </Tooltip>
  )
}
