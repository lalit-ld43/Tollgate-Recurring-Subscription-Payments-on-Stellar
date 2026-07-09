import GateArm from './GateArm'
import NextChargeCountdown from './NextChargeCountdown'
import { EXPLORER_TX_URL } from '../lib/config'

export default function SubscriptionCard({ subscription, plan, onCancel, lastTxHash, busy }) {
  return (
    <article className="bg-asphalt-soft border border-asphalt-line rounded-lg p-4 sm:p-5">
      <div className="flex items-center justify-between gap-3 mb-4">
        <div>
          <p className="font-mono text-[10px] uppercase tracking-widest2 text-chalk-dim/50">
            Your Subscription
          </p>
          <h3 className="font-display font-semibold text-lg text-chalk mt-0.5">
            {plan?.name || `Plan #${subscription.planId}`}
          </h3>
        </div>
        <GateArm status={subscription.status} />
      </div>

      <div className="grid grid-cols-2 gap-3 mb-4 text-xs font-mono">
        <div>
          <p className="text-chalk-dim/50 uppercase tracking-wide text-[10px] mb-0.5">Missed charges</p>
          <p className={subscription.missedCharges > 0 ? 'text-signal-hold' : 'text-chalk-dim'}>
            {subscription.missedCharges}
          </p>
        </div>
        <div>
          <p className="text-chalk-dim/50 uppercase tracking-wide text-[10px] mb-0.5">Price / cycle</p>
          <p className="text-amber">{plan?.price ?? '—'}</p>
        </div>
      </div>

      {subscription.status !== 'Cancelled' && (
        <div className="mb-4">
          <NextChargeCountdown nextDueAt={subscription.nextDueAt} />
        </div>
      )}

      {subscription.status !== 'Cancelled' && (
        <button
          onClick={() => onCancel(subscription.planId)}
          disabled={busy}
          className="w-full font-mono text-sm px-4 py-2.5 rounded border border-signal-stop/50 text-signal-stop hover:bg-signal-stop/10 disabled:opacity-40 transition-colors"
        >
          Cancel subscription
        </button>
      )}

      {lastTxHash && (
        <a
          href={EXPLORER_TX_URL(lastTxHash)}
          target="_blank"
          rel="noreferrer"
          className="block mt-3 font-mono text-[10px] text-chalk-dim/40 hover:text-amber truncate transition-colors"
        >
          last tx: {lastTxHash}
        </a>
      )}
    </article>
  )
}
