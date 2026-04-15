import { Component, type ErrorInfo, type ReactNode } from "react";
import { logger } from "@/lib/logger";

const log = logger.child({ module: "error-boundary" });

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    log.error(
      { err: error, componentStack: info.componentStack },
      "unhandled render error",
    );
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-screen items-center justify-center">
          <div className="text-center space-y-4">
            <h2
              className="text-lg font-semibold"
              style={{ color: "var(--panel-text-primary)" }}
            >
              エラーが発生しました
            </h2>
            <p
              className="text-sm"
              style={{ color: "var(--panel-text-secondary)" }}
            >
              {this.state.error?.message}
            </p>
            <button
              type="button"
              onClick={() => this.setState({ hasError: false, error: null })}
              className="rounded-lg px-4 py-2 text-sm"
              style={{
                backgroundColor: "var(--panel-active-bg)",
                color: "var(--panel-text-primary)",
              }}
            >
              再試行
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
