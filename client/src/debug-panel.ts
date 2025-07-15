import { logger } from "./logger";

export interface DebugPanelOptions {
  position?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
  collapsed?: boolean;
}

export class DebugPanel {
  private container: HTMLDivElement;
  private content: HTMLDivElement;
  private values: Map<string, unknown> = new Map();
  private collapsed = false;

  constructor(options: DebugPanelOptions = {}) {
    const { position = "bottom-left", collapsed = false } = options;
    this.collapsed = collapsed;

    // Create container
    this.container = document.createElement("div");
    this.container.style.cssText = `
      position: fixed;
      ${position.includes("top") ? "top: 10px" : "bottom: 10px"};
      ${position.includes("left") ? "left: 10px" : "right: 10px"};
      background: rgba(0, 0, 0, 0.9);
      color: #fff;
      font-family: monospace;
      font-size: 12px;
      padding: 10px;
      border-radius: 5px;
      border: 1px solid #333;
      min-width: 200px;
      z-index: 1000;
    `;

    // Create header
    const header = document.createElement("div");
    header.style.cssText = `
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 10px;
      cursor: pointer;
    `;
    header.innerHTML = `
      <strong>Debug Panel</strong>
      <span id="debug-toggle">${collapsed ? "▶" : "▼"}</span>
    `;
    header.addEventListener("click", () => this.toggle());

    // Create content area
    this.content = document.createElement("div");
    this.content.style.display = collapsed ? "none" : "block";

    this.container.appendChild(header);
    this.container.appendChild(this.content);
    document.body.appendChild(this.container);

    logger.debug("Debug panel created");
  }

  set(key: string, value: unknown): void {
    this.values.set(key, value);
    this.render();
  }

  remove(key: string): void {
    this.values.delete(key);
    this.render();
  }

  clear(): void {
    this.values.clear();
    this.render();
  }

  toggle(): void {
    this.collapsed = !this.collapsed;
    this.content.style.display = this.collapsed ? "none" : "block";
    const toggle = this.container.querySelector("#debug-toggle");
    if (toggle) {
      toggle.textContent = this.collapsed ? "▶" : "▼";
    }
  }

  destroy(): void {
    this.container.remove();
  }

  private render(): void {
    if (this.collapsed) {
      return;
    }

    const html = Array.from(this.values.entries())
      .map(([key, value]) => {
        const displayValue =
          typeof value === "object"
            ? JSON.stringify(value, null, 2)
            : String(value);
        return `<div><strong>${key}:</strong> ${displayValue}</div>`;
      })
      .join("");

    this.content.innerHTML =
      html || '<div style="color: #888">No debug values</div>';
  }

  // Helper methods for common values
  setConnection(
    status: "disconnected" | "connecting" | "connected",
    details?: string,
  ): void {
    const color = {
      disconnected: "#f44336",
      connecting: "#ff9800",
      connected: "#4caf50",
    }[status];

    this.set(
      "Connection",
      `<span style="color: ${color}">${status}</span>${details !== undefined ? ` (${details})` : ""}`,
    );
  }

  setLatency(ms: number): void {
    const color = ms < 50 ? "#4caf50" : ms < 100 ? "#ff9800" : "#f44336";
    this.set("Latency", `<span style="color: ${color}">${ms}ms</span>`);
  }

  setEntities(count: number): void {
    this.set("Entities", count);
  }

  setBandwidth(bytesPerSecond: number): void {
    const kb = (bytesPerSecond / 1024).toFixed(2);
    this.set("Bandwidth", `${kb} KB/s`);
  }
}

// Export for debugging
(window as unknown as Record<string, unknown>).DebugPanel = DebugPanel;
