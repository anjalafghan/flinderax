import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { UpdateTransactionModal } from '@/components/feature/UpdateTransactionModal'
import api from '@/services/api'

vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
  },
}))

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: false },
    mutations: { retry: false },
  },
})

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
)

describe('UpdateTransactionModal', () => {
  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    cardId: 'test-card-id',
    currentBalance: 1000,
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    queryClient.clear()
  })

  it('renders modal when isOpen is true', () => {
    render(<UpdateTransactionModal {...defaultProps} />, { wrapper })
    expect(screen.getByText('Update Balance')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('0.00')).toBeInTheDocument()
  })

  it('does not render modal when isOpen is false', () => {
    render(<UpdateTransactionModal {...defaultProps} isOpen={false} />, { wrapper })
    expect(screen.queryByText('Update Balance')).not.toBeInTheDocument()
  })

  it('calculates delta correctly when new amount is greater than current', async () => {
    render(<UpdateTransactionModal {...defaultProps} />, { wrapper })
    const input = screen.getByPlaceholderText('0.00')
    fireEvent.change(input, { target: { value: '2000' } })
    expect(screen.getByText('Amount to transfer')).toBeInTheDocument()
    expect(screen.getByText('+₹1,000.00')).toBeInTheDocument()
  })

  it('shows negative delta when new amount is less than current', async () => {
    render(<UpdateTransactionModal {...defaultProps} />, { wrapper })
    const input = screen.getByPlaceholderText('0.00')
    fireEvent.change(input, { target: { value: '500' } })
    expect(screen.getByText('Amount to transfer')).toBeInTheDocument()
    expect(screen.getByText('-₹500.00')).toBeInTheDocument()
  })

  it('shows zero delta when amounts are equal', async () => {
    render(<UpdateTransactionModal {...defaultProps} />, { wrapper })
    const input = screen.getByPlaceholderText('0.00')
    fireEvent.change(input, { target: { value: '1000' } })
    expect(screen.getByText('Amount to transfer')).toBeInTheDocument()
    expect(screen.getByText('₹0.00')).toBeInTheDocument()
  })

  it('sends absolute value to API when confirming transfer via modal', async () => {
    const mockPost = vi.fn().mockResolvedValue({
      data: { transaction_id: 'tx-123', amount_due: 1000, status: true }
    })
    vi.spyOn(api, 'post').mockImplementation(mockPost)

    render(<UpdateTransactionModal {...defaultProps} />, { wrapper })
    const input = screen.getByPlaceholderText('0.00')
    fireEvent.change(input, { target: { value: '2000' } })

    const transferButton = screen.getByText("I've transferred ₹1,000.00")
    fireEvent.click(transferButton)

    await act(async () => {
      const confirmButton = await screen.findByText("I've transferred")
      fireEvent.click(confirmButton)
    })

    await waitFor(() => {
      expect(mockPost).toHaveBeenCalledWith('/card/insert_transaction', {
        card_id: 'test-card-id',
        amount_due: 2000,
      })
    })

    vi.restoreAllMocks()
  })

  it('shows defer payment option', async () => {
    render(<UpdateTransactionModal {...defaultProps} />, { wrapper })
    const input = screen.getByPlaceholderText('0.00')
    fireEvent.change(input, { target: { value: '2000' } })

    expect(screen.getByText('Defer Payment')).toBeInTheDocument()
  })
})
