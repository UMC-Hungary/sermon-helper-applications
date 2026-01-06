export class OBSWebSocket {
	constructor(private socket: WebSocket) {

		obs.call("SetInputSettings", {
			inputName: "00__browser",
			inputSettings: {
				url: `https://nyiregyhazimetodista.hu/obs-caption.html?type=caption&title=&bold=Textus: hello ${1 ? `&light=Lekció: ${1}` : ''}&color=red&showLogo=true`
			}
		})
	}
}

// Export singleton instance
export const obsWebSocket = new OBSWebSocket();