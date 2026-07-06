// Global refresh bus for the topbar button: the button asks the ACTIVE view
// to re-pull its data. Views register a handler once at init (they stay
// mounted for the app's lifetime) and decide themselves what "refresh" means:
// GitHub-backed views force past the disk cache, notes/todos reload from disk.

class RefreshBus {
  /** True while a refresh is running; drives the button spinner. */
  busy = $state(false);
  #handlers = new Map<string, () => Promise<void>>();

  register(view: string, handler: () => Promise<void>) {
    this.#handlers.set(view, handler);
  }

  async run(view: string) {
    const handler = this.#handlers.get(view);
    if (!handler || this.busy) return;
    this.busy = true;
    try {
      await handler();
    } finally {
      this.busy = false;
    }
  }
}

export const refreshBus = new RefreshBus();
