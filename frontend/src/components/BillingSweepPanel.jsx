import { useState } from 'react'

export default function BillingSweepPanel({ onRunCycle, disabled, lastResult }) {
  const [busy, setBusy] = useState(false)

  const handleRun = async () => {
    setBusy(true)
    try {
      await onRunCycle()
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="bg-asphalt-soft border border-asphalt-line rounded-lg p-4 sm:p-5">
      <div className="flex items-center justify-between mb-3">
        <div>
          <h3 className="font-display font-semibold text-base text-chalk">Billing Sweep</h3>
          <p className="font-mono text-[11px] text-chalk-dim/50 mt-0.5">
            Normally run by a cron keeper — trigger manually for the demo
          </p>
        </div>
      </div>

      <button
        onClick={handleRun}
        disabled={busy || disabled}
        className="w-full font-mono text-sm px-4 py-2.5 rounded bg-amber text-asphalt font-medium hover:bg-amber-bright disabled:opacity-40 transition-colors"
      >
        {busy ? 'Sweeping due subscriptions…' : 'Run billing cycle now'}
      </button>

      {lastResult !== null && lastResult !== undefined && (
        <p className="font-mono text-xs text-chalk-dim/60 mt-3 text-center">
          Last sweep charged <span className="text-amber">{lastResult}</span>{' '}
          subscription{lastResult === 1 ? '' : 's'}
        </p>
      )}
    </div>
  )
}
