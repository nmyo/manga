import { LoaderCircleIcon } from 'lucide-react'
import { useEffect, useState } from 'react'

import { cn } from '@/lib/utils'
import { ReaderError } from './reader-state'

export function ReaderImage({ src }: { src: string }) {
  const [displaySrc, setDisplaySrc] = useState('')
  const [status, setStatus] = useState<'loading' | 'loaded' | 'error'>('loading')
  const hasDisplaySrc = displaySrc.length > 0

  useEffect(() => {
    let isActive = true
    const image = new Image()

    setStatus('loading')
    image.onload = () => {
      if (!isActive) {
        return
      }

      setDisplaySrc(src)
      setStatus('loaded')
    }
    image.onerror = () => {
      if (!isActive) {
        return
      }

      setStatus('error')
    }
    image.src = src

    return () => {
      isActive = false
      image.onload = null
      image.onerror = null
    }
  }, [src])

  return (
    <div className="relative flex h-screen w-screen items-center justify-center">
      {status === 'loading' && !hasDisplaySrc ? (
        <div className="absolute inset-0 flex items-center justify-center">
          <LoaderCircleIcon className="size-6 animate-spin text-neutral-400" />
        </div>
      ) : null}
      {status === 'error' ? (
        <ReaderError title="图片显示失败" description="图片文件已生成，但浏览器暂时无法读取。" />
      ) : null}
      {displaySrc.length > 0 ? (
        <img
          src={displaySrc}
          alt=""
          className={cn(
            'relative z-10 h-screen w-screen object-contain transition-opacity',
            status === 'error' ? 'opacity-20' : 'opacity-100'
          )}
          draggable={false}
        />
      ) : null}
    </div>
  )
}
