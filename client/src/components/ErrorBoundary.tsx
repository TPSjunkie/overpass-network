// src/components/ErrorBoundary.tsx

import React, { Component, type ErrorInfo, type ReactNode } from 'react';
import { toast } from 'react-toastify';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
}

class ErrorBoundary extends Component<Props, State> {
  public override state: State = {
    hasError: false,
  };

  public static getDerivedStateFromError(_: Error): State {
    // Update state so the next render shows the fallback UI.
    return { hasError: true };
  }

  public override componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught error:", error, errorInfo);
    toast.error("An unexpected error occurred.");
  }

  public override render() {
    if (this.state.hasError) {
      return <h1 className="text-pip-boy-text text-center mt-20">Something went wrong.</h1>;
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
