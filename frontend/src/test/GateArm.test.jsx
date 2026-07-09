import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import GateArm from '../components/GateArm'

describe('GateArm', () => {
  it('renders ACTIVE label for Active status', () => {
    render(<GateArm status="Active" />)
    expect(screen.getByText('ACTIVE')).toBeInTheDocument()
  })

  it('renders PAST DUE label for PastDue status', () => {
    render(<GateArm status="PastDue" />)
    expect(screen.getByText('PAST DUE')).toBeInTheDocument()
  })

  it('renders CANCELLED label for Cancelled status', () => {
    render(<GateArm status="Cancelled" />)
    expect(screen.getByText('CANCELLED')).toBeInTheDocument()
  })

  it('falls back to Cancelled style for unknown status', () => {
    render(<GateArm status="Weird" />)
    expect(screen.getByText('CANCELLED')).toBeInTheDocument()
  })
})
