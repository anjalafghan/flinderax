import { Component, type ErrorInfo, type ReactNode, } from 'react';
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
                <div className="flex h-screen flex-col items-center justify-center bg-background p-6 text-center text-foreground">
                    <h2 className="mb-4 text-2xl font-bold">Something went wrong</h2>
                    <p className="mb-6 max-w-md text-muted-foreground">
                        {this.state.error?.message || "An unexpected error occurred."}
                    </p>
                    <Button onClick={() => window.location.reload()}>Reload Page</Button>
                    <pre className="mt-8 overflow-auto rounded bg-chip p-4 text-left text-xs text-over max-w-2xl w-full">
                        {this.state.error?.stack}
                    </pre>
                </div>
            );
        }

        return this.props.children;
    }
}
