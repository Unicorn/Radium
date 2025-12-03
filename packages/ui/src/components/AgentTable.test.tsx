import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AgentTable } from './AgentTable';
import type { Agent } from '@radium/shared-types';

// Note: Tests may need adjustments based on actual component implementation

describe('AgentTable', () => {
	const mockAgents: Agent[] = [
		{
			id: 'agent-1',
			name: 'Test Agent 1',
			description: 'First test agent',
			configJson: '{}',
			state: 'idle',
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		},
		{
			id: 'agent-2',
			name: 'Test Agent 2',
			description: 'Second test agent',
			configJson: '{}',
			state: 'running',
			createdAt: new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		},
	];

	it('should render loading state', () => {
		render(<AgentTable agents={[]} loading={true} />);
		expect(screen.getByText('Loading agents...')).toBeInTheDocument();
	});

	it('should render empty state', () => {
		render(<AgentTable agents={[]} loading={false} />);
		expect(screen.getByText('No agents found')).toBeInTheDocument();
	});

	it('should render agents table', () => {
		render(<AgentTable agents={mockAgents} loading={false} />);
		expect(screen.getByText('Test Agent 1')).toBeInTheDocument();
		expect(screen.getByText('Test Agent 2')).toBeInTheDocument();
	});

	it('should call onSelect when row is clicked', () => {
		const onSelect = vi.fn();
		render(
			<AgentTable agents={mockAgents} loading={false} onSelect={onSelect} />
		);
		// Find the table row and click it
		const rows = screen.getAllByRole('row');
		rows[1].click(); // Skip header row
		expect(onSelect).toHaveBeenCalled();
	});

	it('should call onDelete when delete button is clicked', () => {
		const onDelete = vi.fn();
		render(
			<AgentTable agents={mockAgents} loading={false} onDelete={onDelete} />
		);
		const deleteButtons = screen.getAllByText('Delete');
		if (deleteButtons.length > 0) {
			deleteButtons[0].click();
			expect(onDelete).toHaveBeenCalled();
		}
	});
});

