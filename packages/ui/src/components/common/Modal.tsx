import React from 'react';

export interface ModalProps {
	isOpen: boolean;
	onClose: () => void;
	title?: string;
	children: React.ReactNode;
}

export const Modal: React.FC<ModalProps> = ({
	isOpen,
	onClose,
	title,
	children,
}) => {
	if (!isOpen) return null;

	return (
		<div
			className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
			onClick={onClose}
		>
			<div
				className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto"
				onClick={(e) => e.stopPropagation()}
			>
				{title && (
					<div className="flex items-center justify-between p-4 border-b">
						<h2 className="text-xl font-semibold">{title}</h2>
						<button
							onClick={onClose}
							className="text-gray-400 hover:text-gray-600 text-2xl"
						>
							×
						</button>
					</div>
				)}
				<div className="p-4">{children}</div>
			</div>
		</div>
	);
};

