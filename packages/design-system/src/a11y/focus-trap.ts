const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

function focusable(node: HTMLElement): HTMLElement[] {
  return [...node.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
    (el) => el.offsetParent !== null || el === document.activeElement,
  );
}

/**
 * Keeps Tab and Shift+Tab inside `node` while it is mounted, moves focus into it on mount,
 * and returns focus to whatever opened it on destroy.
 */
export function focusTrap(node: HTMLElement) {
  const opener = document.activeElement as HTMLElement | null;

  function onKeydown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;
    const items = focusable(node);
    if (items.length === 0) {
      event.preventDefault();
      return;
    }
    const first = items[0]!;
    const last = items[items.length - 1]!;
    const active = document.activeElement;
    if (event.shiftKey && (active === first || !node.contains(active))) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }

  // Give the caller's own autofocus a chance before we pick a target.
  queueMicrotask(() => {
    if (node.contains(document.activeElement)) return;
    (node.querySelector<HTMLElement>('[autofocus]') ?? focusable(node)[0] ?? node).focus();
  });

  node.addEventListener('keydown', onKeydown);
  return {
    destroy() {
      node.removeEventListener('keydown', onKeydown);
      opener?.focus?.();
    },
  };
}
