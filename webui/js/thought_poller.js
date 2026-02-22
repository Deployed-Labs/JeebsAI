/**
 * JeebsAI Thought Poller
 * Polls the brain for the latest cognitive state.
 * 
 * Usage:
 * import { ThoughtPoller } from './js/thought_poller.js';
 * const poller = new ThoughtPoller('session:123', (thought) => {
 *     console.log("Jeebs is thinking:", thought.internal_monologue);
 * });
 * poller.start();
 */
export class ThoughtPoller {
    constructor(userId, callback, interval = 1000) {
        this.userId = userId;
        this.callback = callback;
        this.interval = interval;
        this.timer = null;
    }

    start() {
        this.poll();
        this.timer = setInterval(() => this.poll(), this.interval);
    }

    stop() {
        if (this.timer) clearInterval(this.timer);
    }

    async poll() {
        try {
            const res = await fetch(`/api/brain/thought/${encodeURIComponent(this.userId)}`);
            const data = await res.json();
            if (data.success && data.thought) {
                this.callback(data.thought);
            }
        } catch (e) {
            console.error("Thought poll error:", e);
        }
    }
}