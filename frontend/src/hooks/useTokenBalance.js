import { useState, useEffect, useCallback } from 'react'
import { readContract, addressToScVal } from '../lib/sorobanClient'
import { TOKEN_CONTRACT_ID } from '../lib/config'

export function useTokenBalance(walletAddress, feed) {
  const [balance, setBalance] = useState(null)
  const [loading, setLoading] = useState(false)

  const fetchBalance = useCallback(async () => {
    if (!walletAddress || !TOKEN_CONTRACT_ID) {
      setBalance(null)
      return
    }
    try {
      setLoading(true)
      const bal = await readContract({
        contractId: TOKEN_CONTRACT_ID,
        method: 'balance',
        args: [addressToScVal(walletAddress)],
        sourcePublicKey: walletAddress,
      })
      // Token balance is an i128 (BigInt). We convert to string and divide by 10^7 for XLM.
      const rawBal = Number(bal) / 10000000
      setBalance(rawBal.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }))
    } catch (err) {
      console.error('Failed to fetch token balance:', err)
      setBalance(null)
    } finally {
      setLoading(false)
    }
  }, [walletAddress])

  // Fetch immediately on mount or wallet address change
  useEffect(() => {
    fetchBalance()
  }, [fetchBalance])

  // Refresh balance when global feed updates (meaning a transaction happened)
  useEffect(() => {
    if (feed && feed.length > 0) {
      fetchBalance()
    }
  }, [feed, fetchBalance])

  return { balance, loading, refresh: fetchBalance }
}
