import { Component, type ErrorInfo, type ReactNode } from "react";
import { Button } from "./button";

interface Props {
  children: ReactNode;
  /** Changes when the person moves to another page (the path), which clears a crash. */
  resetKey: string;
}

/**
 * Catches a crash inside one page so the menu and the tab bar survive and the person can go
 * elsewhere. The whole-app boundary in main.tsx stays as the last resort.
 */
export class RouteBoundary extends Component<Props, { failed: boolean }> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Page crashed:", error, info);
  }

  componentDidUpdate(prev: Props) {
    if (this.state.failed && prev.resetKey !== this.props.resetKey) this.setState({ failed: false });
  }

  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <div role="alert" className="page flex flex-col items-start gap-3">
        <h2 className="font-display text-2xl">This page hit a problem</h2>
        <p className="text-muted-foreground">The rest of the app still works. You can try again or go somewhere else.</p>
        <Button onClick={() => this.setState({ failed: false })}>Try again</Button>
      </div>
    );
  }
}
