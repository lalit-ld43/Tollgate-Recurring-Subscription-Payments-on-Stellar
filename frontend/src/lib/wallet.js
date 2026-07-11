import {
  isConnected,
  setAllowed,
  requestAccess,
  getNetworkDetails as freighterGetNetworkDetails,
  signTransaction as freighterSignTransaction,
} from '@stellar/freighter-api'

const NETWORK_PASSPHRASES = {
  TESTNET: 'Test SDF Network ; September 2015',
  PUBLIC: 'Public Global Stellar Network ; September 2015',
}

export async function isFreighterInstalled() {
  const result = await isConnected()
  return result.isConnected
}

export async function connectWallet() {
  const connected = await isFreighterInstalled()
  if (!connected) {
    throw new Error(
      'Freighter wallet extension not found. Install it from freighter.app to continue.'
    )
  }
  const allowed = await setAllowed()
  if (allowed.error) {
    throw new Error(allowed.error)
  }
  const addressResult = await requestAccess()
  if (addressResult.error) {
    throw new Error(addressResult.error)
  }
  return addressResult.address
}

export async function getNetworkDetails() {
  const connected = await isFreighterInstalled()
  if (!connected) return null
  return freighterGetNetworkDetails()
}

export async function signTransaction(xdr, networkPassphrase) {
  const connected = await isFreighterInstalled()
  if (!connected) {
    throw new Error('Freighter wallet extension not found.')
  }
  const result = await freighterSignTransaction(xdr, {
    networkPassphrase,
  })
  if (result.error) {
    throw new Error(result.error)
  }
  return result.signedTxXdr
}

export { NETWORK_PASSPHRASES }
