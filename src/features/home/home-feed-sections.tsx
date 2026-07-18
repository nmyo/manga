import { ComicGrid } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import type { HomeFeedSection } from '@/lib/api/home'
import { homeSectionId } from './home-utils'

export function HomeFeedSections({ sections }: { sections: HomeFeedSection[] }) {
  return (
    <>
      {sections.map(section => (
        <section key={section.id} id={homeSectionId(section)} className="scroll-mt-8">
          {section.items.length === 0 ? (
            <EmptyState emoji="(･o･;)" title="暂无内容" />
          ) : (
            <ComicGrid items={section.items} />
          )}
        </section>
      ))}
    </>
  )
}
