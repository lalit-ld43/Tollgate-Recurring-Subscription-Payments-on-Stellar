import { useEffect, useState } from 'react'

const STATUS_CONFIG = {
  Active: { raised: true, color: 'bg-signal-go', label: 'ACTIVE' },
  PastDue: { raised: false, color: 'bg-signal-hold', label: 'PAST DUE' },
  Cancelled: { raised: false, color: 'bg-signal-stop', label: 'CANCELLED' },
}

export default function GateArm({ status, size = 'md' }) {
  const config = STATUS_CONFIG[status] || STATUS_CONFIG.Cancelled
  const [animClass, setAnimClass] = useState('')
  const dims = size === 'sm' ? { post: 'h-10', arm: 'w-10' } : { post: 'h-14', arm: 'w-14' }

  useEffect(() => {
    setAnimClass(config.raised ? 'gate-arm-up' : 'gate-arm-down')
  }, [config.raised])

  return (
    <div className="flex items-center gap-2.5">
      <div className={`relative ${dims.post} w-2 rounded-sm bg-asphalt-line shrink-0`}>
        <div
          className={`absolute bottom-0 left-1/2 origin-bottom-left h-1.5 ${dims.arm} rounded-sm hazard-stripes ${animClass}`}
          style={{ transform: config.raised ? 'rotate(-72deg)' : 'rotate(0deg)' }}
        />
      </div>
      <span
        className={`font-mono text-[10px] uppercase tracking-widest2 px-2 py-1 rounded-sm text-asphalt font-semibold ${config.color}`}
      >
        {config.label}
      </span>
    </div>
  )
}
