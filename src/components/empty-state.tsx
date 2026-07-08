interface EmptyStateProps {
  emoji: string
  title: string
  description?: string
  action?: React.ReactNode
}

export function EmptyState({ emoji, title, description, action }: EmptyStateProps) {
  return (
    <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-4 text-center">
      <p className="text-6xl font-bold text-foreground">{emoji}</p>
      <div className="space-y-1">
        <p className="text-sm font-medium text-foreground">{title}</p>
        {description && <p className="text-xs text-muted-foreground">{description}</p>}
      </div>
      {action && <div className="pointer-events-auto">{action}</div>}
    </div>
  )
}
