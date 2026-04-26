import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { CreditCard } from '@/components/feature/CreditCard'

describe('CreditCard', () => {
  const defaultProps = {
    id: 'test-card-id',
    name: 'Test Card',
    bank: 'Test Bank',
    balance: 5000,
    lastDelta: 0,
    primaryColor: [100, 100, 100] as [number, number, number],
    secondaryColor: [200, 200, 200] as [number, number, number],
    last4Digits: '1234',
  }

  it('renders card name and bank correctly', () => {
    render(<CreditCard {...defaultProps} />)
    expect(screen.getByText('Test Card')).toBeInTheDocument()
    expect(screen.getByText('Test Bank')).toBeInTheDocument()
  })

  it('displays Total Due label when balance is positive', () => {
    render(<CreditCard {...defaultProps} balance={1000} />)
    expect(screen.getByText('Total Due')).toBeInTheDocument()
  })

  it('displays Credit Balance label when balance is negative', () => {
    render(<CreditCard {...defaultProps} balance={-500} />)
    expect(screen.getByText('Credit Balance')).toBeInTheDocument()
  })

  it('formats balance as INR currency', () => {
    render(<CreditCard {...defaultProps} balance={5000} />)
    expect(screen.getByText('₹5,000.00')).toBeInTheDocument()
  })

  it('shows last 4 digits correctly', () => {
    render(<CreditCard {...defaultProps} />)
    expect(screen.getByText('•••• •••• •••• 1234')).toBeInTheDocument()
  })

  it('shows masked digits when last4Digits is null', () => {
    render(<CreditCard {...defaultProps} last4Digits={null} />)
    expect(screen.getByText('•••• •••• •••• ••••')).toBeInTheDocument()
  })

  it('displays lastDelta when it is not zero', () => {
    render(<CreditCard {...defaultProps} lastDelta={1000} />)
    expect(screen.getByText('₹1,000.00')).toBeInTheDocument()
  })
})
