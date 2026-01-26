import React, { Component, ErrorInfo, ReactNode } from 'react';
import { Button } from './button';

interface Props {
    children: ReactNode;
}

interface State {
    hasError: boolean;
    error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
        error: null,
    };

    public static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error('Uncaught error:', error, errorInfo);
    }

    public render() {
        if (this.state.hasError) {
            return (
                <div className="flex h-screen flex-col items-center justify-center bg-gray-50 p-6 text-center text-gray-900">
                    <h2 className="mb-4 text-2xl font-bold">Something went wrong</h2>
                    <p className="mb-6 max-w-md text-gray-500">
                        {this.state.error?.message || "An unexpected error occurred."}
                    </p>
                    <Button onClick={() => window.location.reload()}>Reload Page</Button>
                    <pre className="mt-8 overflow-auto rounded bg-gray-100 p-4 text-left text-xs text-red-500 max-w-2xl w-full">
                        {this.state.error?.stack}
                    </pre>
                </div>
            );
        }

        return this.props.children;
    }
}
