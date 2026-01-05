// todo: implement
import type { SystemStatus } from './types';

export const systemStatus = {
	obs: true,
	rodeInterface: true,
	mainDisplay: true,
	secondaryDisplay: true,
	airplayDisplay: false,
	displayAlignment: false,
	youtubeLoggedIn: false,
} satisfies SystemStatus;