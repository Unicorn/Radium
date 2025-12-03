import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Button } from './Button';

describe('Button', () => {
	it('should render button with text', () => {
		render(<Button>Click me</Button>);
		expect(screen.getByText('Click me')).toBeInTheDocument();
	});

	it('should apply primary variant by default', () => {
		render(<Button>Test</Button>);
		const button = screen.getByText('Test');
		expect(button.className).toContain('bg-blue-600');
	});

	it('should apply secondary variant', () => {
		render(<Button variant="secondary">Test</Button>);
		const button = screen.getByText('Test');
		expect(button.className).toContain('bg-gray-200');
	});

	it('should apply danger variant', () => {
		render(<Button variant="danger">Delete</Button>);
		const button = screen.getByText('Delete');
		expect(button.className).toContain('bg-red-600');
	});

	it('should handle onClick', () => {
		const handleClick = vi.fn();
		render(<Button onClick={handleClick}>Click</Button>);
		screen.getByText('Click').click();
		expect(handleClick).toHaveBeenCalledTimes(1);
	});
});

