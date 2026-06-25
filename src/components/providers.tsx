import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from 'next-themes'
import { ReactNode } from 'react'
import { Toaster } from 'sonner'

import { TooltipProvider } from './ui/tooltip'

const queryClient = new QueryClient()

export function Providers({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider attribute="class" enableSystem={true} disableTransitionOnChange>
        <TooltipProvider>{children}</TooltipProvider>
        <Toaster
          toastOptions={{
            classNames: {
              toast: 'font-sans'
            }
          }}
        />
      </ThemeProvider>
    </QueryClientProvider>
  )
}
