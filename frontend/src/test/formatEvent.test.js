import { describe, it, expect } from 'vitest'
import { formatEvent, formatEvents } from '../lib/formatEvent'

describe('formatEvent', () => {
  it('labels a plan created event correctly', () => {
    const event = { id: '1', topics: ['plan', 'created', 0], ledger: 100 }
    const result = formatEvent(event)
    expect(result.label).toBe('Plan Created')
    expect(result.tone).toBe('neutral')
  })

  it('labels a subscription started event with go tone', () => {
    const event = { id: '2', topics: ['sub', 'started', 0], ledger: 101 }
    const result = formatEvent(event)
    expect(result.label).toBe('Subscription Started')
    expect(result.tone).toBe('go')
  })

  it('labels a successful charge event with go tone', () => {
    const event = { id: '3', topics: ['charge', 'ok', 0], ledger: 102 }
    const result = formatEvent(event)
    expect(result.label).toBe('Charge Succeeded')
    expect(result.tone).toBe('go')
  })

  it('labels a failed charge event with stop tone', () => {
    const event = { id: '4', topics: ['charge', 'failed', 0], ledger: 103 }
    const result = formatEvent(event)
    expect(result.label).toBe('Charge Failed')
    expect(result.tone).toBe('stop')
  })

  it('labels a billing cycle sweep with hold tone', () => {
    const event = { id: '5', topics: ['billing', 'cycle'], ledger: 104 }
    const result = formatEvent(event)
    expect(result.label).toBe('Billing Cycle Swept')
    expect(result.tone).toBe('hold')
  })

  it('falls back gracefully for unrecognized topics', () => {
    const event = { id: '6', topics: ['mystery', 'thing'], ledger: 105 }
    const result = formatEvent(event)
    expect(result.label).toBe('mystery / thing')
    expect(result.tone).toBe('neutral')
  })

  it('handles malformed or missing event input without throwing', () => {
    expect(formatEvent(null).label).toBe('Unknown Event')
    expect(formatEvent({}).label).toBe('Unknown Event')
    expect(formatEvent({ topics: null }).label).toBe('Unknown Event')
  })

  it('preserves ledger, txHash, and timestamp metadata', () => {
    const event = { id: '7', topics: ['sub', 'cancel', 2], ledger: 200, txHash: 'deadbeef', timestamp: '2026-07-08T12:00:00Z' }
    const result = formatEvent(event)
    expect(result.ledger).toBe(200)
    expect(result.txHash).toBe('deadbeef')
    expect(result.timestamp).toBe('2026-07-08T12:00:00Z')
  })
})

describe('formatEvents', () => {
  it('maps an array of raw events to formatted entries in order', () => {
    const events = [
      { id: '1', topics: ['plan', 'created'], ledger: 1 },
      { id: '2', topics: ['sub', 'cancel'], ledger: 2 },
    ]
    const results = formatEvents(events)
    expect(results).toHaveLength(2)
    expect(results[0].label).toBe('Plan Created')
    expect(results[1].label).toBe('Subscription Cancelled')
  })

  it('returns an empty array when given no events', () => {
    expect(formatEvents([])).toEqual([])
  })
})
