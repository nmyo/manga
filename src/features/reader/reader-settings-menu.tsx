import { Settings2Icon } from 'lucide-react'
import type { ReactNode } from 'react'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings-store'

const READER_SETTING_BUTTON_CLASS =
  'h-7 rounded-md px-2 text-xs text-neutral-200 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50'

const READER_SETTING_ITEM_CLASS =
  'text-neutral-100 focus:bg-white/10 focus:text-neutral-50 [&_svg]:text-neutral-300'

export function ReaderSettingsMenu() {
  const readerReadMode = useSettingsStore(state => state.readerReadMode)
  const readerPageDirection = useSettingsStore(state => state.readerPageDirection)
  const readerDoublePageMode = useSettingsStore(state => state.readerDoublePageMode)
  const setReaderReadMode = useSettingsStore(state => state.setReaderReadMode)
  const setReaderPageDirection = useSettingsStore(state => state.setReaderPageDirection)
  const setReaderDoublePageMode = useSettingsStore(state => state.setReaderDoublePageMode)
  const isSingleMode = readerReadMode === 'single'

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button type="button" variant="ghost" size="xs" className={READER_SETTING_BUTTON_CLASS}>
          <Settings2Icon className="size-3.5" />
          阅读设置
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        side="top"
        align="start"
        className="w-56 border border-white/10 bg-neutral-950/95 text-neutral-50 shadow-2xl backdrop-blur-xl"
      >
        <DropdownMenuLabel className="text-neutral-400">阅读模式</DropdownMenuLabel>
        <DropdownMenuRadioGroup value={readerReadMode} onValueChange={setReaderReadMode}>
          <DropdownMenuRadioItem value="single" className={READER_SETTING_ITEM_CLASS}>
            单页
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="strip" className={READER_SETTING_ITEM_CLASS}>
            竖向阅读
          </DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
        {isSingleMode ? (
          <>
            <DropdownMenuSeparator className="bg-white/10" />
            <DropdownMenuLabel className="text-neutral-400">翻页方向</DropdownMenuLabel>
            <div className="grid grid-cols-2 gap-1 px-1 pb-1">
              <ReaderDirectionButton
                selected={readerPageDirection === 'ltr'}
                onClick={() => setReaderPageDirection('ltr')}
              >
                从左向右
              </ReaderDirectionButton>
              <ReaderDirectionButton
                selected={readerPageDirection === 'rtl'}
                onClick={() => setReaderPageDirection('rtl')}
              >
                从右向左
              </ReaderDirectionButton>
            </div>
          </>
        ) : null}
        <DropdownMenuSeparator className="bg-white/10" />
        <div className="flex items-center justify-between gap-3 px-3 py-2">
          <div className="min-w-0">
            <div className="text-sm text-neutral-100">双页阅读</div>
            <div className="mt-0.5 text-xs text-neutral-500">仅在单页模式中生效</div>
          </div>
          <Switch
            checked={readerDoublePageMode}
            disabled={!isSingleMode}
            onCheckedChange={setReaderDoublePageMode}
          />
        </div>
        <DropdownMenuSeparator className="bg-white/10" />
        <div className="px-3 py-2 text-xs leading-5 text-neutral-400">
          单页模式可切换左右阅读方向；竖向阅读会纵向连续显示当前章节图片。
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

function ReaderDirectionButton({
  selected,
  onClick,
  children
}: {
  selected: boolean
  onClick: () => void
  children: ReactNode
}) {
  return (
    <button
      type="button"
      className={cn(
        'h-8 rounded-md px-2 text-xs text-neutral-300 transition-colors hover:bg-white/10 hover:text-neutral-50 focus-visible:bg-white/10 focus-visible:text-neutral-50 focus-visible:outline-none',
        selected && 'bg-white/12 text-neutral-50 ring-1 ring-white/15'
      )}
      onClick={onClick}
    >
      {children}
    </button>
  )
}
