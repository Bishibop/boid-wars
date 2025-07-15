use wasm_bindgen::prelude::*;
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement};
use boid_wars_shared::*;

// Game state that persists between frames
#[wasm_bindgen]
pub struct GameClient {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    players: Vec<PlayerState>,
    boids: Vec<BoidState>,
    frame_count: u32,
}

// Simple state structs for visualization
struct PlayerState {
    x: f32,
    y: f32,
    name: String,
}

struct BoidState {
    x: f32,
    y: f32,
    id: u32,
}

#[wasm_bindgen]
impl GameClient {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<GameClient, JsValue> {
        // Set up panic hook for better error messages
        console_error_panic_hook::set_once();
        
        console::log_1(&"🚀 Initializing Boid Wars WASM Client".into());
        
        // Get canvas element
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("Failed to get canvas"))?;
            
        // Get 2D context
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
            
        // Set canvas size
        canvas.set_width(GAME_WIDTH as u32);
        canvas.set_height(GAME_HEIGHT as u32);
        
        console::log_1(&format!("✅ Canvas initialized: {}x{}", GAME_WIDTH, GAME_HEIGHT).into());
        
        // Create initial game state for testing
        let mut players = Vec::new();
        players.push(PlayerState {
            x: 200.0,
            y: 300.0,
            name: "TestPlayer".to_string(),
        });
        
        let mut boids = Vec::new();
        boids.push(BoidState {
            x: 400.0,
            y: 300.0,
            id: 1,
        });
        
        Ok(GameClient {
            canvas,
            context,
            players,
            boids,
            frame_count: 0,
        })
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, delta_ms: f32) {
        // Convert to seconds
        let delta = delta_ms / 1000.0;
        
        // Simple AI: Move boid towards player
        if let Some(boid) = self.boids.get_mut(0) {
            if let Some(player) = self.players.get(0) {
                let dx = player.x - boid.x;
                let dy = player.y - boid.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance > 0.0 {
                    boid.x += (dx / distance) * BOID_SPEED * delta;
                    boid.y += (dy / distance) * BOID_SPEED * delta;
                }
            }
        }
        
        self.frame_count += 1;
    }
    
    #[wasm_bindgen]
    pub fn render(&self) {
        // Clear canvas
        self.context.set_fill_style(&JsValue::from_str("#111111"));
        self.context.fill_rect(0.0, 0.0, GAME_WIDTH as f64, GAME_HEIGHT as f64);
        
        // Draw grid
        self.draw_grid();
        
        // Draw players
        for player in &self.players {
            self.draw_player(player.x as f64, player.y as f64, &player.name);
        }
        
        // Draw boids
        for boid in &self.boids {
            self.draw_boid(boid.x as f64, boid.y as f64);
        }
        
        // Draw HUD
        self.draw_hud();
    }
    
    fn draw_grid(&self) {
        self.context.set_stroke_style(&JsValue::from_str("#333333"));
        self.context.set_line_width(1.0);
        
        // Vertical lines
        for i in 0..=(GAME_WIDTH as i32 / 50) {
            let x = (i * 50) as f64;
            self.context.begin_path();
            self.context.move_to(x, 0.0);
            self.context.line_to(x, GAME_HEIGHT as f64);
            self.context.stroke();
        }
        
        // Horizontal lines
        for i in 0..=(GAME_HEIGHT as i32 / 50) {
            let y = (i * 50) as f64;
            self.context.begin_path();
            self.context.move_to(0.0, y);
            self.context.line_to(GAME_WIDTH as f64, y);
            self.context.stroke();
        }
    }
    
    fn draw_player(&self, x: f64, y: f64, name: &str) {
        // Draw player circle
        self.context.set_fill_style(&JsValue::from_str("#00ff00"));
        self.context.begin_path();
        self.context.arc(x, y, 15.0, 0.0, std::f64::consts::PI * 2.0).unwrap();
        self.context.fill();
        
        // Draw player name
        self.context.set_fill_style(&JsValue::from_str("#ffffff"));
        self.context.set_font("12px monospace");
        self.context.fill_text(name, x - 30.0, y - 20.0).unwrap();
    }
    
    fn draw_boid(&self, x: f64, y: f64) {
        // Draw boid triangle
        self.context.set_fill_style(&JsValue::from_str("#ff0000"));
        self.context.begin_path();
        self.context.move_to(x + 10.0, y);
        self.context.line_to(x - 5.0, y - 5.0);
        self.context.line_to(x - 5.0, y + 5.0);
        self.context.close_path();
        self.context.fill();
    }
    
    fn draw_hud(&self) {
        // Draw FPS counter
        self.context.set_fill_style(&JsValue::from_str("#ffffff"));
        self.context.set_font("14px monospace");
        let info = format!("Frame: {} | Players: {} | Boids: {}", 
                          self.frame_count, self.players.len(), self.boids.len());
        self.context.fill_text(&info, 10.0, 20.0).unwrap();
        
        // Draw server status
        self.context.set_fill_style(&JsValue::from_str("#ffff00"));
        self.context.fill_text("🔴 Offline Mode (No Server Connection)", 10.0, 40.0).unwrap();
    }
    
    #[wasm_bindgen]
    pub fn handle_key_down(&mut self, key: &str) {
        // Simple player movement for testing
        if let Some(player) = self.players.get_mut(0) {
            let speed = 5.0;
            match key {
                "w" | "W" => player.y -= speed,
                "s" | "S" => player.y += speed,
                "a" | "A" => player.x -= speed,
                "d" | "D" => player.x += speed,
                _ => {}
            }
            
            // Keep player in bounds
            player.x = player.x.clamp(0.0, GAME_WIDTH);
            player.y = player.y.clamp(0.0, GAME_HEIGHT);
        }
    }
    
    #[wasm_bindgen]
    pub fn handle_click(&mut self, x: f32, y: f32) {
        console::log_1(&format!("Click at ({}, {})", x, y).into());
        // TODO: Implement shooting
    }
}

// Initialize function called from JavaScript
#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"WASM module loaded successfully!".into());
}