import { useTokenBalance } from '../hooks/useTokenBalance'

export default function Header({ wallet, feed }) {
  const short = (addr) => (addr ? `${addr.slice(0, 4)}…${addr.slice(-4)}` : '')
  const { balance } = useTokenBalance(wallet.address, feed)

  return (
    <header className="border-b border-asphalt-line bg-asphalt/80 backdrop-blur sticky top-0 z-30">
      <div className="max-w-5xl mx-auto px-4 sm:px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded bg-amber/15 border border-amber/40 flex items-center justify-center">
            <span className="font-display font-bold text-amber text-sm leading-none">T</span>
          </div>
          <div>
            <h1 className="font-display font-semibold text-lg sm:text-xl tracking-tight text-chalk leading-none">
              Tollgate
            </h1>
            <p className="font-mono text-[10px] uppercase tracking-widest2 text-chalk-dim/60 leading-none mt-1">
              Recurring Billing · Soroban
            </p>
          </div>
        </div>

        <div>
          {wallet.address ? (
            <div className="flex items-center gap-4">
              {balance !== null && (
                <div className="font-mono text-xs sm:text-sm text-chalk-dim hidden sm:block">
                  {balance} XLM
                </div>
              )}
              <button
                onClick={wallet.disconnect}
                className="font-mono text-xs sm:text-sm px-3 py-2 rounded border border-amber/40 text-amber hover:bg-amber/10 transition-colors"
                title="Click to disconnect"
              >
                {short(wallet.address)}
              </button>
            </div>
          ) : (
            <button
              onClick={wallet.connect}
              disabled={wallet.connecting}
              className="font-mono text-xs sm:text-sm px-3 sm:px-4 py-2 rounded bg-amber text-asphalt font-medium hover:bg-amber-bright transition-colors disabled:opacity-50"
            >
              {wallet.connecting ? 'Connecting…' : 'Connect Wallet'}
            </button>
          )}
        </div>
      </div>
      {!wallet.installed && (
        <div className="bg-signal-stop/10 border-t border-signal-stop/30 text-signal-stop text-xs sm:text-sm text-center py-2 px-4">
          Freighter wallet extension not detected —{' '}
          <a href="https://freighter.app" target="_blank" rel="noreferrer" className="underline">
            install it
          </a>{' '}
          to interact with contracts.
        </div>
      )}
    </header>
  )
}
