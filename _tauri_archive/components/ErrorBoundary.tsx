import { AlertTriangle, RefreshCw } from "lucide-react";
import React, { Component } from "react";

/**
 * 错误边界组件的 Props
 */
interface ErrorBoundaryProps {
	children: React.ReactNode;
	fallback?: React.ReactNode;
	onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
	resetOnPropsChange?: boolean;
	resetKeys?: Array<string | number>;
}

/**
 * 错误边界组件的 State
 */
interface ErrorBoundaryState {
	hasError: boolean;
	error: Error | null;
	errorInfo: React.ErrorInfo | null;
}

/**
 * 错误边界组件
 * 捕获子组件中的 JavaScript 错误，显示备用 UI，防止整个应用崩溃
 */
export class ErrorBoundary extends Component<
	ErrorBoundaryProps,
	ErrorBoundaryState
> {
	private resetTimeoutId: number | null = null;

	constructor(props: ErrorBoundaryProps) {
		super(props);
		this.state = {
			hasError: false,
			error: null,
			errorInfo: null,
		};
	}

	static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
		// 更新 state 使下一次渲染能够显示降级后的 UI
		return { hasError: true, error };
	}

	componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
		// 记录错误信息
		console.error("ErrorBoundary caught an error:", error, errorInfo);

		this.setState({
			error,
			errorInfo,
		});

		// 调用错误回调
		if (this.props.onError) {
			this.props.onError(error, errorInfo);
		}
	}

	componentDidUpdate(prevProps: ErrorBoundaryProps) {
		const { resetOnPropsChange, resetKeys } = this.props;
		const { hasError } = this.state;

		// 如果之前有错误，但是重置键发生了变化，重置错误边界
		if (hasError && resetOnPropsChange && resetKeys) {
			const prevResetKeys = prevProps.resetKeys || [];
			const hasResetKeyChanged = resetKeys.some(
				(key, index) => key !== prevResetKeys[index],
			);

			if (hasResetKeyChanged) {
				this.resetErrorBoundary();
			}
		}
	}

	componentWillUnmount() {
		if (this.resetTimeoutId) {
			clearTimeout(this.resetTimeoutId);
		}
	}

	/**
	 * 重置错误边界
	 */
	private resetErrorBoundary = () => {
		if (this.resetTimeoutId) {
			clearTimeout(this.resetTimeoutId);
			this.resetTimeoutId = null;
		}

		this.setState({
			hasError: false,
			error: null,
			errorInfo: null,
		});
	};

	/**
	 * 手动重试
	 */
	private handleRetry = () => {
		this.resetErrorBoundary();
	};

	render() {
		const { hasError, error } = this.state;
		const { children, fallback } = this.props;

		if (hasError) {
			// 如果提供了自定义 fallback，使用它
			if (fallback) {
				return fallback;
			}

			// 默认错误 UI
			return (
				<div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4">
					<div className="bg-surface rounded-lg shadow-lg p-8 max-w-md w-full">
						<div className="flex items-center space-x-3 mb-4">
							<AlertTriangle className="h-8 w-8 text-red-500" />
							<h2 className="text-xl font-semibold text-gray-900 dark:text-white">
								出现错误
							</h2>
						</div>

						<p className="text-gray-600 dark:text-gray-300 mb-6">
							应用程序遇到了一个错误。这可能是由于窗口调整或其他操作引起的。
						</p>

						{error && (
							<details className="mb-6">
								<summary className="cursor-pointer text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">
									查看错误详情
								</summary>
								<div className="mt-2 p-3 bg-gray-100 dark:bg-gray-700 rounded text-xs font-mono text-gray-800 dark:text-gray-200 overflow-auto max-h-32">
									{error.message}
								</div>
							</details>
						)}

						<div className="flex space-x-3">
							<button
								onClick={this.handleRetry}
								className="flex-1 flex items-center justify-center space-x-2 bg-theme-primary bg-theme-primary-hover text-white font-medium py-2 px-4 rounded-lg theme-transition"
							>
								<RefreshCw className="h-4 w-4" />
								<span>重试</span>
							</button>

							<button
								onClick={() => window.location.reload()}
								className="flex-1 bg-gray-200 hover:bg-gray-300 dark:bg-gray-600 dark:hover:bg-gray-500 text-gray-800 dark:text-white font-medium py-2 px-4 rounded-lg transition-colors"
							>
								刷新页面
							</button>
						</div>

						<p className="text-xs text-gray-500 dark:text-gray-400 mt-4 text-center">
							如果问题持续存在，请尝试重启应用程序
						</p>
					</div>
				</div>
			);
		}

		return children;
	}
}

/**
 * 高阶组件：为组件添加错误边界
 */
export function withErrorBoundary<P extends object>(
	Component: React.ComponentType<P>,
	errorBoundaryProps?: Omit<ErrorBoundaryProps, "children">,
) {
	const WrappedComponent = (props: P) => (
		<ErrorBoundary {...errorBoundaryProps}>
			<Component {...props} />
		</ErrorBoundary>
	);

	WrappedComponent.displayName = `withErrorBoundary(${Component.displayName || Component.name})`;

	return WrappedComponent;
}

/**
 * React Hook：错误边界状态管理
 */
export const useErrorHandler = () => {
	const [error, setError] = React.useState<Error | null>(null);

	const resetError = React.useCallback(() => {
		setError(null);
	}, []);

	const handleError = React.useCallback((error: Error) => {
		console.error("Handled error:", error);
		setError(error);
	}, []);

	// 如果有错误，抛出它以便错误边界捕获
	if (error) {
		throw error;
	}

	return { handleError, resetError };
};
