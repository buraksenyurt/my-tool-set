// A shared, one-second clock for "running for 2 min" labels. Ticks only while
// something is subscribed.

class Clock {
  now = $state(Date.now());
  private users = 0;
  private timer: ReturnType<typeof setInterval> | null = null;

  /** Call from an $effect; returns the cleanup. */
  use(): () => void {
    if (this.users++ === 0) {
      this.now = Date.now();
      this.timer = setInterval(() => (this.now = Date.now()), 1000);
    }
    return () => {
      if (--this.users === 0 && this.timer) {
        clearInterval(this.timer);
        this.timer = null;
      }
    };
  }
}

export const clock = new Clock();
