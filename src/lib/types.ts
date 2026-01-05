export interface SystemStatus {
	obs: {
		connected: boolean;
		rodeInterface?: boolean;
		mainDisplay?: boolean;
		secondaryDisplay?: boolean;
		airplayDisplay?: boolean;
		displayAlignment?: boolean;
	};
	youtube: {
		authenticated: boolean;
		loggedIn?: boolean;
	};
	bible: {
		loaded: boolean;
	};
}

export interface YoutubeEvent {
	id: string;
	title: string;
	scheduledStartTime: string;
	status: string;
}

export interface ScheduleItem {
	id: string;
	title: string;
	date: string;
	time: string;
	description?: string;
}

export interface Sermon {
	id: string;
	title: string;
	textus: string;
	lectionary: string;
	content: string;
}