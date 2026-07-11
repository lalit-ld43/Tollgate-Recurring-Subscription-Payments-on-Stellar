const NETWORK_PASSPHRASES = {
  TESTNET: 'Test SDF Network ; September 2015',
  PUBLIC: 'Public Global Stellar Network ; September 2015',
}

export function isFreighterInstalled() {
  return typeof window !== 'undefined' && (!!window.freighterApi || !!window.freighter)
}

export async function connectWallet() {
  if (!isFreighterInstalled()) {
    throw new Error(
      'Freighter wallet extension not found. Install it from freighter.app to continue.'
    )
  }
  const api = window.freighterApi || window.freighter;
  const access = await api.requestAccess()
  if (access.error) {
    throw new Error(access.error)
  }
  const addressResult = await api.getAddress()
  if (addressResult.error) {
    throw new Error(addressResult.error)
  }
  return addressResult.address
}

export async function getNetworkDetails() {
  if (!isFreighterInstalled()) return null
  const api = window.freighterApi || window.freighter;
  return api.getNetworkDetails()
}

export async function signTransaction(xdr, networkPassphrase) {
  if (!isFreighterInstalled()) {
    throw new Error('Freighter wallet extension not found.')
  }
  const api = window.freighterApi || window.freighter;
  const result = await api.signTransaction(xdr, {
    networkPassphrase,
  })
  if (result.error) {
    throw new Error(result.error)
  }
  return result.signedTxXdr
}

export { NETWORK_PASSPHRASES }
