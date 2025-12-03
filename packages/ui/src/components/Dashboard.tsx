import React from 'react';

export interface DashboardStats {
	agentsCount: number;
	workflowsCount: number;
	tasksCount: number;
	activeTasksCount: number;
}

export interface DashboardProps {
	stats: DashboardStats;
	onNavigate?: (section: string) => void;
}

export const Dashboard: React.FC<DashboardProps> = ({ stats, onNavigate }) => {
	const cards = [
		{
			title: 'Agents',
			count: stats.agentsCount,
			color: 'bg-blue-500',
			onClick: () => onNavigate?.('agents'),
		},
		{
			title: 'Workflows',
			count: stats.workflowsCount,
			color: 'bg-green-500',
			onClick: () => onNavigate?.('workflows'),
		},
		{
			title: 'Tasks',
			count: stats.tasksCount,
			color: 'bg-yellow-500',
			onClick: () => onNavigate?.('tasks'),
		},
		{
			title: 'Active Tasks',
			count: stats.activeTasksCount,
			color: 'bg-purple-500',
			onClick: () => onNavigate?.('tasks'),
		},
	];

	return (
		<div className="p-6">
			<h1 className="text-3xl font-bold mb-6">Dashboard</h1>
			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
				{cards.map((card) => (
					<div
						key={card.title}
						className={`${card.color} rounded-lg shadow-lg p-6 text-white cursor-pointer hover:opacity-90 transition-opacity`}
						onClick={card.onClick}
					>
						<h3 className="text-lg font-semibold mb-2">{card.title}</h3>
						<p className="text-4xl font-bold">{card.count}</p>
					</div>
				))}
			</div>
		</div>
	);
};

