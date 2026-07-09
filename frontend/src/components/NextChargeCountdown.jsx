import { useCountdown } from '../hooks/useCountdown'

export default function NextChargeCountdown({ nextDueAt }) {
  const { label, overdue } = useCountdown(nextDueAt)

  return (
    <div className="flex items-center gap-1.5">
      <span
        className={`w-1.5 h-1.5 rounded-full ${overdue ? 'bg-signal-stop animate-pulse' : 'bg-amber'}`}
      />
      <span className="font-mono text-xs text-chalk-dim/70">
        {overdue ? 'Charge due now' : `Next charge in ${label}`}
      </span>
    </div>
  )
}
