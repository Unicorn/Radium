/**
 * React Error Boundary Component
 * Catches errors in component tree and displays user-friendly error messages
 */

'use client';

import React, { Component, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home, ChevronDown, ChevronUp } from 'lucide-react';

interface Props {
  children: ReactNode;
  fallback?: (error: Error, errorInfo: React.ErrorInfo, reset: () => void) => ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
  showDetails: boolean;
}

/**
 * Error Boundary Component
 * Wraps components to catch and handle runtime errors gracefully
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      showDetails: false,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // Log error to console (server-side logging should be done separately)
    console.error('Error caught by boundary:', error, errorInfo);

    // Update state with error info
    this.setState({
      errorInfo,
    });

    // Call optional error handler
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }

    // Log to monitoring service (if configured)
    if (typeof window !== 'undefined' && (window as any).errorLogger) {
      (window as any).errorLogger.error('React Error Boundary caught error', {
        error: error.message,
        stack: error.stack,
        componentStack: errorInfo.componentStack,
      });
    }
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      showDetails: false,
    });
  };

  handleToggleDetails = () => {
    this.setState((prev) => ({
      showDetails: !prev.showDetails,
    }));
  };

  render() {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback && this.state.error && this.state.errorInfo) {
        return this.props.fallback(
          this.state.error,
          this.state.errorInfo,
          this.handleReset
        );
      }

      // Default error UI
      return (
        <DefaultErrorFallback
          error={this.state.error}
          errorInfo={this.state.errorInfo}
          showDetails={this.state.showDetails}
          onReset={this.handleReset}
          onToggleDetails={this.handleToggleDetails}
        />
      );
    }

    return this.props.children;
  }
}

/**
 * Default Error Fallback UI
 */
interface FallbackProps {
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
  showDetails: boolean;
  onReset: () => void;
  onToggleDetails: () => void;
}

function DefaultErrorFallback({
  error,
  errorInfo,
  showDetails,
  onReset,
  onToggleDetails,
}: FallbackProps) {
  return (
    <div className="min-h-[400px] flex items-center justify-center p-4">
      <div className="max-w-2xl w-full bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-red-200 dark:border-red-800">
        {/* Header */}
        <div className="p-6 border-b border-red-200 dark:border-red-800">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-red-100 dark:bg-red-900/30 rounded-lg">
              <AlertTriangle className="w-6 h-6 text-red-600 dark:text-red-400" />
            </div>
            <div>
              <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
                Something went wrong
              </h2>
              <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                An unexpected error occurred while rendering this component
              </p>
            </div>
          </div>
        </div>

        {/* Error Message */}
        <div className="p-6 space-y-4">
          {error && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
              <p className="text-sm font-medium text-red-900 dark:text-red-100">
                Error: {error.message}
              </p>
            </div>
          )}

          {/* Recovery Suggestions */}
          <div className="space-y-2">
            <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100">
              What you can try:
            </h3>
            <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
              <li className="flex items-start gap-2">
                <span className="text-blue-600 dark:text-blue-400">•</span>
                <span>Refresh the page to reload the component</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-blue-600 dark:text-blue-400">•</span>
                <span>Check your internet connection</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-blue-600 dark:text-blue-400">•</span>
                <span>Clear your browser cache and reload</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-blue-600 dark:text-blue-400">•</span>
                <span>If the problem persists, contact support</span>
              </li>
            </ul>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-3 pt-2">
            <button
              onClick={onReset}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
            >
              <RefreshCw className="w-4 h-4" />
              Try Again
            </button>
            <button
              onClick={() => (window.location.href = '/')}
              className="flex items-center gap-2 px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded-lg transition-colors text-sm font-medium"
            >
              <Home className="w-4 h-4" />
              Go Home
            </button>
          </div>

          {/* Technical Details (Collapsible) */}
          {(error || errorInfo) && (
            <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
              <button
                onClick={onToggleDetails}
                className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100 transition-colors"
              >
                {showDetails ? (
                  <ChevronUp className="w-4 h-4" />
                ) : (
                  <ChevronDown className="w-4 h-4" />
                )}
                {showDetails ? 'Hide' : 'Show'} Technical Details
              </button>

              {showDetails && (
                <div className="mt-3 space-y-3">
                  {error?.stack && (
                    <div>
                      <p className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Error Stack:
                      </p>
                      <pre className="text-xs bg-gray-100 dark:bg-gray-900 p-3 rounded border border-gray-300 dark:border-gray-700 overflow-x-auto text-gray-800 dark:text-gray-200 font-mono">
                        {error.stack}
                      </pre>
                    </div>
                  )}

                  {errorInfo?.componentStack && (
                    <div>
                      <p className="text-xs font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Component Stack:
                      </p>
                      <pre className="text-xs bg-gray-100 dark:bg-gray-900 p-3 rounded border border-gray-300 dark:border-gray-700 overflow-x-auto text-gray-800 dark:text-gray-200 font-mono">
                        {errorInfo.componentStack}
                      </pre>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

/**
 * Workflow-specific Error Boundary
 * Customized for workflow builder errors
 */
export function WorkflowErrorBoundary({ children }: { children: ReactNode }) {
  return (
    <ErrorBoundary
      fallback={(error, errorInfo, reset) => (
        <div className="min-h-[400px] flex items-center justify-center p-4">
          <div className="max-w-2xl w-full bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-red-200 dark:border-red-800">
            <div className="p-6 border-b border-red-200 dark:border-red-800">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-red-100 dark:bg-red-900/30 rounded-lg">
                  <AlertTriangle className="w-6 h-6 text-red-600 dark:text-red-400" />
                </div>
                <div>
                  <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
                    Workflow Error
                  </h2>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    The workflow builder encountered an error
                  </p>
                </div>
              </div>
            </div>

            <div className="p-6 space-y-4">
              <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                <p className="text-sm font-medium text-red-900 dark:text-red-100">
                  {error.message}
                </p>
              </div>

              <div className="space-y-2">
                <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  What might have caused this:
                </h3>
                <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
                  <li className="flex items-start gap-2">
                    <span className="text-blue-600 dark:text-blue-400">•</span>
                    <span>Invalid workflow configuration</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-blue-600 dark:text-blue-400">•</span>
                    <span>Missing or disconnected nodes</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="text-blue-600 dark:text-blue-400">•</span>
                    <span>Incompatible node connections</span>
                  </li>
                </ul>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={reset}
                  className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
                >
                  <RefreshCw className="w-4 h-4" />
                  Reset Workflow
                </button>
                <button
                  onClick={() => (window.location.href = '/workflows')}
                  className="flex items-center gap-2 px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100 rounded-lg transition-colors text-sm font-medium"
                >
                  <Home className="w-4 h-4" />
                  Back to Workflows
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
      onError={(error, errorInfo) => {
        // Log workflow-specific errors
        console.error('Workflow error:', error);
        console.error('Component stack:', errorInfo.componentStack);
      }}
    >
      {children}
    </ErrorBoundary>
  );
}

/**
 * Hook to programmatically trigger error boundary
 */
export function useErrorBoundary() {
  const [, setError] = React.useState();

  return React.useCallback((error: Error) => {
    setError(() => {
      throw error;
    });
  }, []);
}
