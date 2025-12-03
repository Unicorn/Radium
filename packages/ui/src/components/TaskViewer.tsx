import React from 'react';
import type { Task } from '@radium/shared-types';

export interface TaskViewerProps {
	task: Task;
	onClose?: () => void;
}

export const TaskViewer: React.FC<TaskViewerProps> = ({ task, onClose }) => {
	return (
		<div className="space-y-4">
			<div className="flex items-center justify-between">
				<h2 className="text-2xl font-bold">{task.name}</h2>
				{onClose && (
					<button
						onClick={onClose}
						className="text-gray-400 hover:text-gray-600 text-2xl"
					>
						×
					</button>
				)}
			</div>

			<div>
				<label className="block text-sm font-medium text-gray-700">
					Description
				</label>
				<p className="mt-1 text-sm text-gray-900">{task.description}</p>
			</div>

			<div className="grid grid-cols-2 gap-4">
				<div>
					<label className="block text-sm font-medium text-gray-700">
						Agent ID
					</label>
					<p className="mt-1 text-sm text-gray-900">{task.agentId}</p>
				</div>
				<div>
					<label className="block text-sm font-medium text-gray-700">State</label>
					<span className="mt-1 inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-blue-100 text-blue-800">
						{task.state}
					</span>
				</div>
			</div>

			<div>
				<label className="block text-sm font-medium text-gray-700">Input</label>
				<pre className="mt-1 p-3 bg-gray-100 rounded text-sm overflow-auto">
					{task.inputJson}
				</pre>
			</div>

			{task.resultJson && (
				<div>
					<label className="block text-sm font-medium text-gray-700">
						Result
					</label>
					<pre className="mt-1 p-3 bg-gray-100 rounded text-sm overflow-auto">
						{task.resultJson}
					</pre>
				</div>
			)}

			<div className="grid grid-cols-2 gap-4 text-sm text-gray-500">
				<div>
					<label className="block font-medium">Created</label>
					<p>{new Date(task.createdAt).toLocaleString()}</p>
				</div>
				<div>
					<label className="block font-medium">Updated</label>
					<p>{new Date(task.updatedAt).toLocaleString()}</p>
				</div>
			</div>
		</div>
	);
};

