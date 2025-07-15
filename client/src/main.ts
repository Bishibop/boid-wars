import { Application, Graphics, Text } from "pixi.js";
import { logger } from "./logger";
import { perfMonitor } from "./perf";

// Status element
const statusEl = document.getElementById("status")!;

logger.info("Boid Wars Client Starting...");

// Initialize Pixi.js
const app = new Application({
  width: 800,
  height: 600,
  backgroundColor: 0x1a1a1a,
  antialias: true,
});

// Add canvas to DOM
document.getElementById("game-container")!.appendChild(app.canvas);

// Add performance monitor
if (import.meta.env.DEV) {
  perfMonitor.createDisplay(document.body);
}

// Create a simple circle that will move when connected
const circle = new Graphics();
circle.beginFill(0x00ff00);
circle.drawCircle(0, 0, 20);
circle.endFill();
circle.x = app.screen.width / 2;
circle.y = app.screen.height / 2;
app.stage.addChild(circle);

// Add connection status text
const connectionText = new Text("Waiting for WASM...", {
  fill: 0xffffff,
  fontSize: 14,
});
connectionText.x = 10;
connectionText.y = 10;
app.stage.addChild(connectionText);

// Animation loop
let time = 0;
app.ticker.add(() => {
  time += app.ticker.deltaTime * 0.01;
  // Simple animation to show rendering works
  circle.y = app.screen.height / 2 + Math.sin(time) * 50;

  // Update performance monitor
  perfMonitor.update();
  perfMonitor.setEntities(app.stage.children.length);
});

// Update status
statusEl.textContent = "Client initialized - Pixi.js running";
statusEl.className = "connected";

// Placeholder for WASM integration
function initializeWasm(): void {
  try {
    // This will be replaced with actual WASM module import
    logger.debug("WASM integration will go here");
    connectionText.text = "WASM not yet integrated";
  } catch (error) {
    logger.error("Failed to initialize WASM:", error);
    connectionText.text = "WASM initialization failed";
  }
}

// Initialize WASM when ready
initializeWasm();

// Export for debugging
(window as unknown as Record<string, unknown>).pixiApp = app;
