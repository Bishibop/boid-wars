import { logger } from "./logger";
import { perfMonitor } from "./perf";
import initWasm, { GameClient } from "./wasm/boid_wars_wasm";

// Status element
const statusEl = document.getElementById("status")!;

logger.info("Boid Wars Client Starting...");

// Game state
let gameClient: GameClient | null = null;
let lastTime = performance.now();
let animationId: number | null = null;

// Add performance monitor
if (import.meta.env.DEV) {
  perfMonitor.createDisplay(document.body);
}

// Initialize canvas
const canvas = document.createElement("canvas");
canvas.id = "game-canvas";
canvas.style.display = "block";
document.getElementById("game-container")!.appendChild(canvas);

// Initialize WASM
async function initializeWasm(): Promise<void> {
  try {
    statusEl.textContent = "Loading WASM...";
    statusEl.className = "loading";
    
    await initWasm();
    logger.info("WASM module loaded");
    
    // Create game client
    gameClient = new GameClient("game-canvas");
    logger.info("Game client created");
    
    // Set up event handlers
    setupEventHandlers();
    
    // Start game loop
    startGameLoop();
    
    statusEl.textContent = "Client running - Offline Mode";
    statusEl.className = "connected";
  } catch (error) {
    logger.error("Failed to initialize WASM:", error);
    statusEl.textContent = "WASM initialization failed";
    statusEl.className = "error";
  }
}

function setupEventHandlers(): void {
  // Keyboard input
  document.addEventListener("keydown", (event) => {
    if (gameClient && !event.repeat) {
      gameClient.handle_key_down(event.key);
    }
  });
  
  // Mouse click
  canvas.addEventListener("click", (event) => {
    if (gameClient) {
      const rect = canvas.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;
      gameClient.handle_click(x, y);
    }
  });
}

function startGameLoop(): void {
  function frame(currentTime: number): void {
    const deltaMs = currentTime - lastTime;
    lastTime = currentTime;
    
    if (gameClient) {
      // Update game state
      gameClient.update(deltaMs);
      
      // Render
      gameClient.render();
      
      // Update performance monitor
      perfMonitor.update();
      perfMonitor.setEntities(2); // 1 player + 1 boid for now
    }
    
    animationId = requestAnimationFrame(frame);
  }
  
  animationId = requestAnimationFrame(frame);
}

// Clean up on page unload
window.addEventListener("beforeunload", () => {
  if (animationId !== null) {
    cancelAnimationFrame(animationId);
  }
});

// Initialize when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => initializeWasm());
} else {
  initializeWasm();
}

// Export for debugging
(window as unknown as Record<string, unknown>).gameClient = gameClient;