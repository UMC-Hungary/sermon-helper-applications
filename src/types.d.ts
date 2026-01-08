interface PromiseConstructor {
	new<T>(executor: (resolve: (value: T) => void, reject?: (reason?: any) => void): Promise<T>;
}

declare var Promise: PromiseConstructor;