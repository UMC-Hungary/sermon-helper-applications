declare module 'mdns-js' {
	interface Service {
		addresses?: string[]
		port?: number
		fullname?: string
		type?: unknown[]
	}

	interface Browser {
		on(event: 'ready', callback: () => void): void
		on(event: 'update', callback: (service: Service) => void): void
		discover(): void
		stop(): void
	}

	interface ServiceType {
		name: string
		protocol: string
	}

	export function tcp(name: string): ServiceType
	export function createBrowser(serviceType: ServiceType): Browser
}
