import React from 'react';
import type { Agent } from '@radium/shared-types';
import { Button } from './common/Button';

export interface AgentTableProps {
	agents: Agent[];
	onSelect?: (agent: Agent) => void;
	onEdit?: (agent: Agent) => void;
	onDelete?: (agentId: string) => void;
	loading?: boolean;
}

export const AgentTable: React.FC<AgentTableProps> = ({
	agents,
	onSelect,
	onEdit,
	onDelete,
	loading = false,
}) => {
	if (loading) {
		return <div className="text-center py-8">Loading agents...</div>;
	}

	if (agents.length === 0) {
		return <div className="text-center py-8 text-gray-500">No agents found</div>;
	}

	return (
		<div className="overflow-x-auto">
			<table className="min-w-full divide-y divide-gray-200">
				<thead className="bg-gray-50">
					<tr>
						<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Name
						</th>
						<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Description
						</th>
						<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							State
						</th>
						<th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Created
						</th>
						<th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
							Actions
						</th>
					</tr>
				</thead>
				<tbody className="bg-white divide-y divide-gray-200">
					{agents.map((agent) => (
						<tr
							key={agent.id}
							className="hover:bg-gray-50 cursor-pointer"
							onClick={() => onSelect?.(agent)}
						>
							<td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
								{agent.name}
							</td>
							<td className="px-6 py-4 text-sm text-gray-500">
								{agent.description}
							</td>
							<td className="px-6 py-4 whitespace-nowrap">
								<span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
									{agent.state}
								</span>
							</td>
							<td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
								{new Date(agent.createdAt).toLocaleDateString()}
							</td>
							<td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
								<div className="flex justify-end gap-2">
									{onEdit && (
										<Button
											variant="secondary"
											size="small"
											onClick={(e) => {
												e.stopPropagation();
												onEdit(agent);
											}}
										>
											Edit
										</Button>
									)}
									{onDelete && (
										<Button
											variant="danger"
											size="small"
											onClick={(e) => {
												e.stopPropagation();
												onDelete(agent.id);
											}}
										>
											Delete
										</Button>
									)}
								</div>
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	);
};

