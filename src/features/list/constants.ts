import { type FilterOption } from '@/lib/ranking-filters'

export const WEEK_OPTIONS: FilterOption[] = [
  { label: '周一', value: '1' },
  { label: '周二', value: '2' },
  { label: '周三', value: '3' },
  { label: '周四', value: '4' },
  { label: '周五', value: '5' },
  { label: '周六', value: '6' },
  { label: '周日', value: '7' },
  { label: '已完结', value: '0' }
]

export const WEEK_CATEGORY_OPTIONS: FilterOption[] = [
  { label: '全部', value: 'all' },
  { label: '日漫', value: 'manga' },
  { label: '韩漫', value: 'hanman' }
]
