/**
 * Svelte action that automatically scales the font size of `node` so its
 * content fits within its parent container without overflowing.
 *
 * Usage:
 *   <div class="slide-text" use:fitText>{text}</div>
 *
 * The action re-runs on every ResizeObserver event (parent resize OR content
 * change via `update()`), binary-searching for the largest font size that
 * does not cause the element to overflow its parent.
 */
/**
 * Pass the text content (or any reactive value) as the action parameter so
 * Svelte calls `update()` whenever the content changes, triggering a re-fit.
 *
 *   <div use:fitText={textContent}> ... </div>
 */
export function fitText(node: HTMLElement, _content?: string): { update: (_content?: string) => void; destroy: () => void } {
	let observer: ResizeObserver;

	function fit() {
		const parent = node.parentElement;
		if (!parent) return;

		const parentH = parent.clientHeight;
		const parentW = parent.clientWidth;
		if (parentH === 0 || parentW === 0) return;

		// Binary search: find the largest font size that fits.
		let lo = 8;
		let hi = 300;

		node.style.fontSize = `${hi}px`;
		while (hi - lo > 1) {
			const mid = Math.floor((lo + hi) / 2);
			node.style.fontSize = `${mid}px`;
			if (node.scrollHeight > parentH || node.scrollWidth > parentW) {
				hi = mid;
			} else {
				lo = mid;
			}
		}
		node.style.fontSize = `${lo}px`;
	}

	observer = new ResizeObserver(fit);
	observer.observe(node);
	if (node.parentElement) {
		observer.observe(node.parentElement);
	}
	fit();

	return {
		update(_newContent?: string) {
			// Called by Svelte when reactive data feeding the element changes.
			fit();
		},
		destroy() {
			observer.disconnect();
		},
	};
}
