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

		// Subtract padding so text doesn't overflow into the padding area.
		const style = getComputedStyle(parent);
		const paddingH = parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
		const paddingW = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight);
		const parentH = parent.clientHeight - paddingH;
		const parentW = parent.clientWidth - paddingW;
		if (parentH <= 0 || parentW <= 0) return;

		// scrollHeight/scrollWidth are integers; parentH/parentW can be
		// fractional (e.g. 281.6 from vw-based padding). Use ceil so that an
		// element whose true width equals the container isn't falsely flagged as
		// overflowing — otherwise the binary search collapses to lo=8.
		const maxH = Math.ceil(parentH);
		const maxW = Math.ceil(parentW);

		// Binary search: find the largest font size that fits.
		// Upper bound scales with the container so 4K/projector screens work.
		let lo = 8;
		let hi = Math.max(600, Math.min(Math.floor(parentW), Math.floor(parentH)));

		node.style.fontSize = `${hi}px`;
		while (hi - lo > 1) {
			const mid = Math.floor((lo + hi) / 2);
			node.style.fontSize = `${mid}px`;
			if (node.scrollHeight > maxH || node.scrollWidth > maxW) {
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
