import { useEffect, useRef, useState } from 'react'

import type { HomeFeedSection } from '@/lib/api/home'
import { cn } from '@/lib/utils'
import { homeSectionId, scrollToHomeSection } from './home-utils'

export function HomeFeedDirectory({ sections }: { sections: HomeFeedSection[] }) {
  const sectionIds = sections.map(homeSectionId)
  const [activeSectionId, setActiveSectionId] = useActiveHomeSection(sectionIds)

  if (sections.length <= 1) return null

  return (
    <nav className="group fixed top-1/2 right-0 z-40 -translate-y-1/2 hidden md:block">
      <div className="flex w-10 flex-col items-end gap-0.5 py-3 pr-3">
        {sections.map(section => {
          const sectionId = homeSectionId(section)
          const isActive = activeSectionId === sectionId

          return (
            <a
              key={section.id}
              href={`#${sectionId}`}
              aria-label={section.title}
              onClick={event => {
                event.preventDefault()
                event.currentTarget.blur()
                setActiveSectionId(sectionId)
                scrollToHomeSection(sectionId)
              }}
              className="flex h-3 w-6 items-center justify-end"
            >
              <span
                className={cn(
                  'h-0.5 rounded-full bg-muted-foreground/35 transition-all duration-200',
                  isActive ? 'w-5 bg-primary' : 'w-2.5',
                  'group-focus-within:w-5 group-hover:w-5 hover:bg-primary'
                )}
              />
            </a>
          )
        })}
      </div>
    </nav>
  )
}

function useActiveHomeSection(sectionIds: string[]) {
  const [activeSectionId, setActiveSectionId] = useState(sectionIds[0] ?? '')
  const visibleSectionsRef = useRef(new Map<string, number>())

  useEffect(() => {
    setActiveSectionId(current =>
      current !== '' && sectionIds.includes(current) ? current : (sectionIds[0] ?? '')
    )
    visibleSectionsRef.current.clear()
  }, [sectionIds])

  useEffect(() => {
    if (sectionIds.length === 0) {
      return
    }

    const observer = new IntersectionObserver(
      entries => {
        for (const entry of entries) {
          const sectionId = entry.target.id

          if (!sectionIds.includes(sectionId)) {
            continue
          }

          if (entry.isIntersecting) {
            visibleSectionsRef.current.set(sectionId, entry.boundingClientRect.top)
          } else {
            visibleSectionsRef.current.delete(sectionId)
          }
        }

        const nextActiveSectionId = [...visibleSectionsRef.current.entries()].sort(
          (left, right) => Math.abs(left[1] - 120) - Math.abs(right[1] - 120)
        )[0]?.[0]

        if (nextActiveSectionId) {
          setActiveSectionId(nextActiveSectionId)
        }
      },
      {
        root: null,
        rootMargin: '-96px 0px -60% 0px',
        threshold: [0, 0.1, 0.35, 0.6, 1]
      }
    )

    for (const sectionId of sectionIds) {
      const element = document.getElementById(sectionId)

      if (element) {
        observer.observe(element)
      }
    }

    return () => observer.disconnect()
  }, [sectionIds])

  return [activeSectionId, setActiveSectionId] as const
}
