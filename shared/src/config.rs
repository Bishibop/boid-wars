use std::env;

/// Network configuration shared between client and server
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub server_bind_addr: String,  // Server binds to this address
    pub client_connect_addr: String, // Client connects to this address
    pub protocol_id: u64,
    pub dev_key: [u8; 32],
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_bind_addr: env::var("BOID_WARS_SERVER_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            client_connect_addr: env::var("BOID_WARS_CLIENT_CONNECT_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            protocol_id: env::var("BOID_WARS_PROTOCOL_ID")
                .unwrap_or_else(|_| "12345".to_string())
                .parse()
                .unwrap_or(12345),
            dev_key: parse_dev_key(),
        }
    }
}

/// Game configuration shared between client and server
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub game_width: f32,
    pub game_height: f32,
    pub player_speed: f32,
    pub boid_speed: f32,
    pub default_health: f32,
    pub spawn_x: f32,
    pub spawn_y: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            game_width: env::var("BOID_WARS_GAME_WIDTH")
                .unwrap_or_else(|_| "1200.0".to_string())
                .parse()
                .unwrap_or(1200.0),
            game_height: env::var("BOID_WARS_GAME_HEIGHT")
                .unwrap_or_else(|_| "900.0".to_string())
                .parse()
                .unwrap_or(900.0),
            player_speed: env::var("BOID_WARS_PLAYER_SPEED")
                .unwrap_or_else(|_| "200.0".to_string())
                .parse()
                .unwrap_or(200.0),
            boid_speed: env::var("BOID_WARS_BOID_SPEED")
                .unwrap_or_else(|_| "150.0".to_string())
                .parse()
                .unwrap_or(150.0),
            default_health: env::var("BOID_WARS_DEFAULT_HEALTH")
                .unwrap_or_else(|_| "100.0".to_string())
                .parse()
                .unwrap_or(100.0),
            spawn_x: env::var("BOID_WARS_SPAWN_X")
                .unwrap_or_else(|_| "600.0".to_string())
                .parse()
                .unwrap_or(600.0),
            spawn_y: env::var("BOID_WARS_SPAWN_Y")
                .unwrap_or_else(|_| "450.0".to_string())
                .parse()
                .unwrap_or(450.0),
        }
    }
}

/// Server-specific configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub status_log_interval: f32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            status_log_interval: env::var("BOID_WARS_STATUS_LOG_INTERVAL")
                .unwrap_or_else(|_| "5.0".to_string())
                .parse()
                .unwrap_or(5.0),
        }
    }
}

/// Client-specific configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub performance_log_interval: f32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            performance_log_interval: env::var("BOID_WARS_PERFORMANCE_LOG_INTERVAL")
                .unwrap_or_else(|_| "5.0".to_string())
                .parse()
                .unwrap_or(5.0),
        }
    }
}

/// Parse development key from environment or use secure default
fn parse_dev_key() -> [u8; 32] {
    if let Ok(key_str) = env::var("BOID_WARS_DEV_KEY") {
        // Try to parse as hex string
        if key_str.len() == 64 {
            let mut key = [0u8; 32];
            if hex::decode_to_slice(&key_str, &mut key).is_ok() {
                return key;
            }
        }
        // Try to parse as comma-separated bytes
        if let Ok(bytes) = key_str
            .split(',')
            .map(|s| s.trim().parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
        {
            if bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                return key;
            }
        }
    }

    // Default secure dev key (better than all zeros)
    [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ]
}

// Remove circular reference - LazyLock instances below provide global access

// Lazy static instances for global access
use std::sync::LazyLock;

pub static NETWORK_CONFIG: LazyLock<NetworkConfig> = LazyLock::new(NetworkConfig::default);
pub static GAME_CONFIG: LazyLock<GameConfig> = LazyLock::new(GameConfig::default);
pub static SERVER_CONFIG: LazyLock<ServerConfig> = LazyLock::new(ServerConfig::default);
pub static CLIENT_CONFIG: LazyLock<ClientConfig> = LazyLock::new(ClientConfig::default);
