import { useState } from 'react'
import { TOKEN_CONTRACT_ID } from '../lib/config'

const DEFAULT_TOKEN = TOKEN_CONTRACT_ID || 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC'

export default function CreatePlanForm({ onCreate, disabled }) {
  const [open, setOpen] = useState(false)
  const [name, setName] = useState('')
  const [token, setToken] = useState(DEFAULT_TOKEN)
  const [price, setPrice] = useState('')
  const [periodDays, setPeriodDays] = useState('30')
  const [busy, setBusy] = useState(false)

  const valid = name.trim() && token.trim() && Number(price) > 0 && Number(periodDays) > 0

  const handleSubmit = async (e) => {
    e.preventDefault()
    if (!valid) return
    setBusy(true)
    try {
      await onCreate({
        name: name.trim(),
        token: token.trim(),
        price: Number(price),
        periodSeconds: Number(periodDays) * 86400,
      })
      setName('')
      setToken(DEFAULT_TOKEN)
      setPrice('')
      setPeriodDays('30')
      setOpen(false)
    } finally {
      setBusy(false)
    }
  }

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        disabled={disabled}
        className="w-full py-4 border-2 border-dashed border-asphalt-line rounded-lg text-chalk-dim/60 hover:border-amber/50 hover:text-amber transition-colors font-mono text-sm disabled:opacity-40 disabled:cursor-not-allowed"
      >
        + Create a new plan
      </button>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="bg-asphalt-soft border border-amber/30 rounded-lg p-4 sm:p-5 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-display font-semibold text-lg text-chalk">New Plan</h3>
        <button type="button" onClick={() => setOpen(false)} className="font-mono text-xs text-chalk-dim/50 hover:text-chalk">
          cancel
        </button>
      </div>

      <div>
        <label className="font-mono text-[10px] uppercase tracking-widest2 text-chalk-dim/60 block mb-1.5">
          Plan name
        </label>
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Pro Monthly"
          className="w-full bg-asphalt border border-asphalt-line rounded p-2.5 text-sm text-chalk placeholder:text-chalk-dim/30 focus:border-amber/60 outline-none"
        />
      </div>

      <div>
        <label className="font-mono text-[10px] uppercase tracking-widest2 text-chalk-dim/60 block mb-1.5">
          Payment token contract
        </label>
        <input
          value={token}
          onChange={(e) => setToken(e.target.value)}
          placeholder="C..."
          className="w-full bg-asphalt border border-asphalt-line rounded p-2.5 text-sm font-mono text-chalk placeholder:text-chalk-dim/30 focus:border-amber/60 outline-none"
        />
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="font-mono text-[10px] uppercase tracking-widest2 text-chalk-dim/60 block mb-1.5">
            Price per cycle
          </label>
          <input
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            type="number"
            min="0"
            placeholder="1000"
            className="w-full bg-asphalt border border-asphalt-line rounded p-2.5 text-sm font-mono text-chalk placeholder:text-chalk-dim/30 focus:border-amber/60 outline-none"
          />
        </div>
        <div>
          <label className="font-mono text-[10px] uppercase tracking-widest2 text-chalk-dim/60 block mb-1.5">
            Billing period (days)
          </label>
          <input
            value={periodDays}
            onChange={(e) => setPeriodDays(e.target.value)}
            type="number"
            min="1"
            placeholder="30"
            className="w-full bg-asphalt border border-asphalt-line rounded p-2.5 text-sm font-mono text-chalk placeholder:text-chalk-dim/30 focus:border-amber/60 outline-none"
          />
        </div>
      </div>

      <button
        type="submit"
        disabled={!valid || busy || disabled}
        className="w-full font-mono text-sm px-4 py-2.5 rounded bg-amber text-asphalt font-medium hover:bg-amber-bright disabled:opacity-40 transition-colors"
      >
        {busy ? 'Creating…' : 'Create plan'}
      </button>
    </form>
  )
}
