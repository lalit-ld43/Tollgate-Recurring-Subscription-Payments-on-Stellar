import { useState, useCallback, useEffect } from 'react'
import Header from './components/Header'
import Hero from './components/Hero'
import CreatePlanForm from './components/CreatePlanForm'
import PlanCard from './components/PlanCard'
import SubscriptionCard from './components/SubscriptionCard'
import BillingSweepPanel from './components/BillingSweepPanel'
import ActivityFeed from './components/ActivityFeed'
import CardSkeleton from './components/CardSkeleton'
import Banner from './components/Banner'
import { useWallet } from './hooks/useWallet'
import { useEventStream } from './hooks/useEventStream'
import { isConfigured } from './lib/config'
import {
  createPlan,
  subscribe,
  cancelSubscription,
  registerWithBilling,
  runBillingCycle,
  getPlan,
  getSubscription,
  normalizePlan,
  normalizeSubscription,
} from './lib/subscriptionActions'

export default function App() {
  const wallet = useWallet()
  const { feed, isPolling } = useEventStream()

  const [plans, setPlans] = useState({})
  const [knownPlanIds, setKnownPlanIds] = useState([])
  const [mySubscriptions, setMySubscriptions] = useState({})
  const [txByPlan, setTxByPlan] = useState({})
  const [lastSweepResult, setLastSweepResult] = useState(null)
  const [errorMsg, setErrorMsg] = useState(null)

  const configured = isConfigured()

  const refreshPlan = useCallback(async (planId) => {
    try {
      const raw = await getPlan({ planId, sourcePublicKey: wallet.address })
      setPlans((prev) => ({ ...prev, [planId]: normalizePlan(planId, raw) }))
    } catch (err) {
      console.error(`Failed to load plan ${planId}:`, err)
    }
  }, [wallet.address])

  const refreshSubscription = useCallback(async (planId) => {
    if (!wallet.address) return
    try {
      const raw = await getSubscription({
        subscriber: wallet.address,
        planId,
        sourcePublicKey: wallet.address,
      })
      const normalized = normalizeSubscription(raw)
      if (normalized) {
        setMySubscriptions((prev) => ({ ...prev, [planId]: normalized }))
      }
    } catch {
      // No subscription for this plan yet — expected, not an error.
    }
  }, [wallet.address])

  // React to live events: refresh whichever plan/subscription changed.
  useEffect(() => {
    if (!configured || feed.length === 0) return
    const planIds = new Set()
    feed.forEach((entry) => {
      const rawTopics = entry.raw?.topics || []
      const numericTopic = rawTopics.find((t) => typeof t === 'number' || typeof t === 'bigint')
      if (numericTopic !== undefined) planIds.add(Number(numericTopic))
    })
    planIds.forEach((id) => {
      refreshPlan(id)
      refreshSubscription(id)
      setKnownPlanIds((prev) => (prev.includes(id) ? prev : [...prev, id]))
    })
  }, [feed, configured, refreshPlan, refreshSubscription])

  const handleCreatePlan = async ({ name, token, price, periodSeconds }) => {
    if (!wallet.address) {
      setErrorMsg('Connect your wallet before creating a plan.')
      return
    }
    setErrorMsg(null)
    try {
      const { result: planId, hash } = await createPlan({
        merchant: wallet.address,
        token,
        price,
        periodSeconds,
        name,
      })
      setTxByPlan((prev) => ({ ...prev, [planId]: hash }))
      setKnownPlanIds((prev) => [...prev, Number(planId)])
      await refreshPlan(Number(planId))
    } catch (err) {
      setErrorMsg(err.message || 'Failed to create plan.')
    }
  }

  const handleSubscribe = async (planId) => {
    if (!wallet.address) {
      setErrorMsg('Connect your wallet to subscribe.')
      return
    }
    try {
      const { hash } = await subscribe({ subscriber: wallet.address, planId })
      setTxByPlan((prev) => ({ ...prev, [planId]: hash }))
      await refreshSubscription(planId)
      // Auto-register with billing so the sweep picks it up.
      await registerWithBilling({ caller: wallet.address, subscriber: wallet.address, planId })
    } catch (err) {
      setErrorMsg(err.message || 'Failed to subscribe.')
    }
  }

  const handleCancel = async (planId) => {
    try {
      const { hash } = await cancelSubscription({ subscriber: wallet.address, planId })
      setTxByPlan((prev) => ({ ...prev, [planId]: hash }))
      await refreshSubscription(planId)
    } catch (err) {
      setErrorMsg(err.message || 'Failed to cancel subscription.')
    }
  }

  const handleRunCycle = async () => {
    if (!wallet.address) {
      setErrorMsg('Connect your wallet to trigger a billing sweep.')
      return
    }
    try {
      const { result, hash } = await runBillingCycle({ caller: wallet.address })
      setLastSweepResult(Number(result))
      setTxByPlan((prev) => ({ ...prev, __sweep: hash }))
      knownPlanIds.forEach((id) => refreshSubscription(id))
    } catch (err) {
      setErrorMsg(err.message || 'Billing sweep failed.')
    }
  }

  const planList = knownPlanIds
    .map((id) => plans[id])
    .filter(Boolean)
    .sort((a, b) => b.id - a.id)

  const subscribedPlanIds = Object.keys(mySubscriptions).map(Number)

  return (
    <div className="min-h-screen flex flex-col">
      <Header wallet={wallet} />
      <Hero />

      <main className="max-w-5xl mx-auto w-full px-4 sm:px-6 pb-16 flex-1">
        {!configured && (
          <div className="mb-6">
            <Banner type="warning">
              Contract addresses aren't configured yet. Set{' '}
              <code className="font-mono">VITE_SUBSCRIPTION_CONTRACT_ID</code>,{' '}
              <code className="font-mono">VITE_BILLING_CONTRACT_ID</code>, and{' '}
              <code className="font-mono">VITE_TOKEN_CONTRACT_ID</code> in your{' '}
              <code className="font-mono">.env</code> file after deploying — see DEPLOYMENT.md.
            </Banner>
          </div>
        )}

        {errorMsg && (
          <div className="mb-6">
            <Banner type="error" onDismiss={() => setErrorMsg(null)}>
              {errorMsg}
            </Banner>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-[1fr_320px] gap-6">
          <section className="space-y-4 min-w-0">
            {subscribedPlanIds.length > 0 && (
              <div className="space-y-3">
                <h3 className="font-mono text-[11px] uppercase tracking-widest2 text-chalk-dim/50">
                  Your subscriptions
                </h3>
                {subscribedPlanIds.map((planId) => (
                  <SubscriptionCard
                    key={planId}
                    subscription={mySubscriptions[planId]}
                    plan={plans[planId]}
                    onCancel={handleCancel}
                    lastTxHash={txByPlan[planId]}
                  />
                ))}
              </div>
            )}

            <BillingSweepPanel
              onRunCycle={handleRunCycle}
              disabled={!wallet.address || !configured}
              lastResult={lastSweepResult}
            />

            <CreatePlanForm onCreate={handleCreatePlan} disabled={!wallet.address || !configured} />

            <h3 className="font-mono text-[11px] uppercase tracking-widest2 text-chalk-dim/50 pt-2">
              Available plans
            </h3>

            {planList.length === 0 && (
              <p className="font-mono text-xs text-chalk-dim/40 text-center py-10">
                No plans yet. The first one created becomes Plan #000.
              </p>
            )}

            {planList.map((plan) => (
              <PlanCard
                key={plan.id}
                plan={plan}
                walletAddress={wallet.address}
                onSubscribe={handleSubscribe}
              />
            ))}
          </section>

          <div className="lg:sticky lg:top-24 lg:self-start">
            <ActivityFeed feed={feed} isPolling={isPolling} />
          </div>
        </div>
      </main>

      <footer className="border-t border-asphalt-line py-6 text-center">
        <p className="font-mono text-[11px] text-chalk-dim/30">Built on Soroban · Stellar Testnet</p>
      </footer>
    </div>
  )
}
