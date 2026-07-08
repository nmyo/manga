import { Trash2Icon } from 'lucide-react'

import { ConfirmDialog } from '@/components/confirm-dialog'
import { Button } from '@/components/ui/button'
import {
  AlertDialog,
  AlertDialogTrigger
} from '@/components/ui/alert-dialog'

interface DeleteSelectedHistoryDialogProps {
  count: number
  disabled: boolean
  onConfirm: () => void
}

export function DeleteSelectedHistoryDialog({
  count,
  disabled,
  onConfirm
}: DeleteSelectedHistoryDialogProps) {
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button type="button" variant="destructive" size="sm" disabled={disabled}>
          <Trash2Icon className="size-4" />
          删除选中
        </Button>
      </AlertDialogTrigger>
      <ConfirmDialog
        open={false}
        onOpenChange={() => {}}
        title="删除阅读记录"
        description={`这会删除选中的 ${count} 条本地阅读进度，删除后无法恢复。`}
        confirmText="确认删除"
        variant="destructive"
        onConfirm={onConfirm}
      />
    </AlertDialog>
  )
}

interface ClearHistoryDialogProps {
  disabled: boolean
  onConfirm: () => void
}

export function ClearHistoryDialog({ disabled, onConfirm }: ClearHistoryDialogProps) {
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button type="button" variant="destructive" size="sm" disabled={disabled}>
          <Trash2Icon className="size-4" />
          清除记录
        </Button>
      </AlertDialogTrigger>
      <ConfirmDialog
        open={false}
        onOpenChange={() => {}}
        title="清除阅读记录"
        description="这会删除本地保存的全部阅读进度，清除后无法恢复。"
        confirmText="确认清除"
        variant="destructive"
        onConfirm={onConfirm}
      />
    </AlertDialog>
  )
}
