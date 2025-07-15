// Performance monitoring utilities

export interface PerfStats {
  fps: number;
  frameTime: number;
  drawCalls: number;
  entities: number;
  networkLatency: number;
}

export class PerfMonitor {
  private stats: PerfStats = {
    fps: 0,
    frameTime: 0,
    drawCalls: 0,
    entities: 0,
    networkLatency: 0,
  };

  private frameCount = 0;
  private lastTime = performance.now();
  private fpsUpdateInterval = 1000; // Update FPS every second

  update(): void {
    const now = performance.now();
    const delta = now - this.lastTime;

    this.frameCount++;

    if (delta >= this.fpsUpdateInterval) {
      this.stats.fps = Math.round((this.frameCount * 1000) / delta);
      this.stats.frameTime = delta / this.frameCount;
      this.frameCount = 0;
      this.lastTime = now;
    }
  }

  setDrawCalls(count: number): void {
    this.stats.drawCalls = count;
  }

  setEntities(count: number): void {
    this.stats.entities = count;
  }

  setNetworkLatency(ms: number): void {
    this.stats.networkLatency = ms;
  }

  getStats(): Readonly<PerfStats> {
    return this.stats;
  }

  createDisplay(container: HTMLElement): HTMLDivElement {
    const display = document.createElement("div");
    display.id = "perf-display";
    display.style.cssText = `
      position: fixed;
      top: 10px;
      right: 10px;
      background: rgba(0, 0, 0, 0.8);
      color: #0f0;
      font-family: monospace;
      font-size: 12px;
      padding: 10px;
      border-radius: 5px;
      pointer-events: none;
      z-index: 1000;
    `;
    container.appendChild(display);

    // Update display
    setInterval(() => {
      display.innerHTML = `
        FPS: ${this.stats.fps}<br>
        Frame: ${this.stats.frameTime.toFixed(2)}ms<br>
        Draws: ${this.stats.drawCalls}<br>
        Entities: ${this.stats.entities}<br>
        Ping: ${this.stats.networkLatency}ms
      `;
    }, 100);

    return display;
  }
}

export const perfMonitor = new PerfMonitor();

// Export for debugging
(window as unknown as Record<string, unknown>).perfMonitor = perfMonitor;
