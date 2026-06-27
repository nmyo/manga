import type { SignInRecord } from '@/lib/api/user'

export function findTodayRecord(records: SignInRecord[]) {
  const today = new Date().getDate()

  return records.find(record => record.day === today)
}
