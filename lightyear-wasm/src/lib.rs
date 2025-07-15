use boid_wars_shared::*;
use wasm_bindgen::prelude::*;
use web_sys::console;

// Lightweight networking client - rendering handled by TypeScript/Pixi.js
// Note: wasm_bindgen has restrictions on complex types - keep API simple
#[wasm_bindgen]
pub struct NetworkClient {
    // TODO: Add Lightyear client connection here
    _placeholder: u32,
}

impl Default for NetworkClient {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl NetworkClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> NetworkClient {
        // Set up panic hook for better error messages
        console_error_panic_hook::set_once();

        console::log_1(&"🌐 Initializing Boid Wars Network Client".into());

        // TODO: Initialize Lightyear client connection here

        NetworkClient { _placeholder: 0 }
    }

    #[wasm_bindgen]
    pub fn connect(&mut self, server_url: &str) -> Result<(), JsValue> {
        console::log_1(&format!("🔗 Connecting to server: {server_url}").into());

        // TODO: Implement Lightyear client connection

        Ok(())
    }

    #[wasm_bindgen]
    pub fn disconnect(&mut self) {
        console::log_1(&"⚡ Disconnecting from server".into());

        // TODO: Implement disconnect
    }

    #[wasm_bindgen]
    pub fn send_input(
        &mut self,
        movement_x: f32,
        movement_y: f32,
        aim_x: f32,
        aim_y: f32,
        fire: bool,
    ) {
        // TODO: Send PlayerInput to server via Lightyear
        let _input = PlayerInput {
            movement: Vec2::new(movement_x, movement_y),
            aim: Vec2::new(aim_x, aim_y),
            fire,
        };
    }

    #[wasm_bindgen]
    pub fn get_player_count(&self) -> u32 {
        // TODO: Get actual player count from Lightyear client
        1
    }

    #[wasm_bindgen]
    pub fn get_boid_count(&self) -> u32 {
        // TODO: Get actual boid count from Lightyear client
        1
    }

    #[wasm_bindgen]
    pub fn is_connected(&self) -> bool {
        // TODO: Get actual connection status from Lightyear client
        false
    }
}

// Initialize function called from JavaScript
#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"🌐 Boid Wars Network Client WASM module loaded".into());
}
