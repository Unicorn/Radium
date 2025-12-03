import React from 'react';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
	variant?: 'primary' | 'secondary' | 'danger';
	size?: 'small' | 'medium' | 'large';
	children: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
	variant = 'primary',
	size = 'medium',
	children,
	className = '',
	...props
}) => {
	const baseStyles = 'font-medium rounded transition-colors';
	const variantStyles = {
		primary: 'bg-blue-600 text-white hover:bg-blue-700',
		secondary: 'bg-gray-200 text-gray-800 hover:bg-gray-300',
		danger: 'bg-red-600 text-white hover:bg-red-700',
	};
	const sizeStyles = {
		small: 'px-2 py-1 text-sm',
		medium: 'px-4 py-2 text-base',
		large: 'px-6 py-3 text-lg',
	};

	return (
		<button
			className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${className}`}
			{...props}
		>
			{children}
		</button>
	);
};

