export default function Hero() {
  return (
    <section className="max-w-5xl mx-auto px-4 sm:px-6 pt-10 sm:pt-16 pb-8 sm:pb-10">
      <p className="font-mono text-[11px] uppercase tracking-widest2 text-amber mb-4">
        Billing that runs itself
      </p>
      <h2 className="font-display font-semibold text-3xl sm:text-5xl leading-[1.05] text-chalk max-w-2xl">
        Subscriptions that charge{' '}
        <span className="text-amber">on schedule</span>, on-chain — no
        backend cron job required.
      </h2>
      <p className="mt-5 text-chalk-dim/80 max-w-xl text-sm sm:text-base leading-relaxed">
        A merchant sets a plan and price. A subscriber opts in once. From
        then on, a separate Billing contract sweeps every due subscription
        and charges it automatically — with a grace period before it gives
        up and cancels.
      </p>
    </section>
  )
}
