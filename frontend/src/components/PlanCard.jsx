export default function PlanCard({ plan, walletAddress, onSubscribe, busy }) {
  const short = (addr) => (addr ? `${addr.slice(0, 6)}…${addr.slice(-4)}` : '—')
  const durationText = plan.periodSeconds >= 86400 
    ? `${Math.round(plan.periodSeconds / 86400)}d` 
    : `${plan.periodSeconds}s`
  const isOwnPlan = walletAddress && plan.merchant === walletAddress

  return (
    <article className="bg-asphalt-soft border border-asphalt-line rounded-lg shadow-gate p-4 sm:p-5">
      <div className="flex items-start justify-between gap-3 mb-3">
        <div>
          <p className="font-mono text-[10px] uppercase tracking-widest2 text-chalk-dim/50">
            Plan #{String(plan.id).padStart(3, '0')}
          </p>
          <h3 className="font-display font-semibold text-lg text-chalk mt-0.5">{plan.name}</h3>
        </div>
        {!plan.active && (
          <span className="px-2 py-1 rounded-sm border border-signal-stop/50 text-signal-stop text-[10px] font-mono uppercase tracking-widest2">
            inactive
          </span>
        )}
      </div>

      <div className="flex items-baseline gap-1 mb-3">
        <span className="font-display font-semibold text-2xl text-amber">{plan.price}</span>
        <span className="font-mono text-xs text-chalk-dim/60">/ {durationText} cycle</span>
      </div>

      <p className="font-mono text-xs text-chalk-dim/50 mb-4">Merchant: {short(plan.merchant)}</p>

      {!isOwnPlan && plan.active && (
        <button
          onClick={() => onSubscribe(plan.id)}
          disabled={busy || !walletAddress}
          className="w-full font-mono text-sm px-4 py-2.5 rounded bg-amber text-asphalt font-medium hover:bg-amber-bright disabled:opacity-40 transition-colors"
        >
          Subscribe
        </button>
      )}
    </article>
  )
}
