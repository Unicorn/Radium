import React, { useState } from 'react';
import type { Workflow, WorkflowStep } from '@radium/shared-types';
import { Button } from './common/Button';
import { Input } from './common/Input';

export interface WorkflowEditorProps {
	workflow?: Workflow;
	onSave: (workflow: Workflow) => void;
	onCancel?: () => void;
}

export const WorkflowEditor: React.FC<WorkflowEditorProps> = ({
	workflow,
	onSave,
	onCancel,
}) => {
	const [name, setName] = useState(workflow?.name || '');
	const [description, setDescription] = useState(workflow?.description || '');
	const [steps, setSteps] = useState<WorkflowStep[]>(workflow?.steps || []);

	const handleSave = () => {
		const updatedWorkflow: Workflow = {
			id: workflow?.id || '',
			name,
			description,
			steps,
			state: workflow?.state || 'draft',
			createdAt: workflow?.createdAt || new Date().toISOString(),
			updatedAt: new Date().toISOString(),
		};
		onSave(updatedWorkflow);
	};

	const addStep = () => {
		const newStep: WorkflowStep = {
			id: `step-${Date.now()}`,
			name: '',
			description: '',
			taskId: '',
			configJson: '{}',
			order: steps.length,
		};
		setSteps([...steps, newStep]);
	};

	const updateStep = (index: number, step: WorkflowStep) => {
		const updated = [...steps];
		updated[index] = step;
		setSteps(updated);
	};

	const removeStep = (index: number) => {
		setSteps(steps.filter((_, i) => i !== index));
	};

	return (
		<div className="space-y-4">
			<Input
				label="Name"
				value={name}
				onChange={(e) => setName(e.target.value)}
				placeholder="Workflow name"
			/>
			<Input
				label="Description"
				value={description}
				onChange={(e) => setDescription(e.target.value)}
				placeholder="Workflow description"
			/>

			<div>
				<div className="flex items-center justify-between mb-2">
					<label className="block text-sm font-medium text-gray-700">
						Steps
					</label>
					<Button size="small" onClick={addStep}>
						Add Step
					</Button>
				</div>
				<div className="space-y-2">
					{steps.map((step, index) => (
						<div key={step.id} className="border rounded p-3">
							<div className="flex items-center justify-between mb-2">
								<span className="text-sm font-medium">Step {index + 1}</span>
								<Button
									variant="danger"
									size="small"
									onClick={() => removeStep(index)}
								>
									Remove
								</Button>
							</div>
							<div className="space-y-2">
								<Input
									label="Step Name"
									value={step.name}
									onChange={(e) =>
										updateStep(index, { ...step, name: e.target.value })
									}
								/>
								<Input
									label="Task ID"
									value={step.taskId}
									onChange={(e) =>
										updateStep(index, { ...step, taskId: e.target.value })
									}
								/>
							</div>
						</div>
					))}
				</div>
			</div>

			<div className="flex gap-2 justify-end">
				{onCancel && (
					<Button variant="secondary" onClick={onCancel}>
						Cancel
					</Button>
				)}
				<Button onClick={handleSave}>Save Workflow</Button>
			</div>
		</div>
	);
};

