// ============================================================
// CANVAS - Ultimate 2D Game Framework for Mintas
// ============================================================
// Easy game development with sprites, collision, physics, and more!
// Uses x,y coordinate grid system for intuitive game creation.

use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex; // Added Mutex

#[cfg(feature = "canvas")]
use minifb::{Key, Window, WindowOptions};
#[cfg(feature = "canvas")]
use image::{GenericImageView, DynamicImage, Rgba}; // Image crate
#[cfg(feature = "canvas")]
use rodio::{OutputStream, Sink, Decoder}; // Audio crate

// Global state - only when canvas feature is enabled
#[cfg(feature = "canvas")]
static mut CANVAS_STATE: Option<CanvasState> = None;
#[cfg(feature = "canvas")]
static mut SPRITES: Option<HashMap<String, Sprite>> = None;
#[cfg(feature = "canvas")]
static mut TEXTURES: Option<HashMap<String, Texture>> = None; // Store textures
// Global State Variables - Required for Canvas
static SCREEN_WIDTH: AtomicU32 = AtomicU32::new(800);
static SCREEN_HEIGHT: AtomicU32 = AtomicU32::new(600);
static GAME_RUNNING: AtomicBool = AtomicBool::new(false);
static SPRITE_COUNTER: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "canvas")]
static mut FRAME_COUNT: u64 = 0;
#[cfg(feature = "canvas")]
static mut LAST_TIME: Option<std::time::Instant> = None;
#[cfg(feature = "canvas")]
static mut DELTA_TIME: f64 = 0.0;
#[cfg(feature = "canvas")]
// Audio wrapper to satisfy Send
struct AudioWrapper(Option<(OutputStream, rodio::OutputStreamHandle)>);
unsafe impl Send for AudioWrapper {}
unsafe impl Sync for AudioWrapper {}

lazy_static::lazy_static! {
    static ref AUDIO_STREAM: Mutex<AudioWrapper> = Mutex::new(AudioWrapper(None));
}

#[cfg(feature = "canvas")]
struct CanvasState {
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    window: Option<Window>,
    keys_down: HashMap<String, bool>,
    keys_pressed: HashMap<String, bool>,
    mouse_x: f64,
    mouse_y: f64,
    mouse_down: [bool; 3],
    mouse_clicked: [bool; 3],
    camera_x: f64,
    camera_y: f64,
    // Audio sinks for fire-and-forget sounds
    sinks: Vec<Sink>,
}

#[cfg(feature = "canvas")]
struct Texture {
    width: u32,
    height: u32,
    pixels: Vec<u32>, // argb format for minifb
}

// Sprite - The core game object
#[cfg(feature = "canvas")]
#[derive(Clone)]
struct Sprite {
    id: String,
    x: f64, y: f64,
    width: f64, height: f64,
    vx: f64, vy: f64,
    ax: f64, ay: f64,  // acceleration
    color: u32,
    visible: bool,
    solid: bool,
    gravity: f64,
    friction: f64,
    on_ground: bool,
    tag: String,
    health: f64,
    speed: f64,
    angle: f64,
    scale: f64,
    flip_x: bool,
    flip_y: bool,
    data: HashMap<String, f64>,  // Custom data storage
    // New Features
    texture: String,
    frame_x: i32,
    frame_y: i32,
    frame_w: i32,
    frame_h: i32,
    opacity: f64,
    z_index: i32,
}

#[cfg(feature = "canvas")]
impl Sprite {
    fn new(id: String, x: f64, y: f64, w: f64, h: f64, color: u32) -> Self {
        Self {
            id, x, y, width: w, height: h,
            vx: 0.0, vy: 0.0, ax: 0.0, ay: 0.0,
            color, visible: true, solid: true,
            gravity: 0.0, friction: 1.0, on_ground: false,
            tag: String::new(), health: 100.0, speed: 5.0,
            angle: 0.0, scale: 1.0, flip_x: false, flip_y: false,
            data: HashMap::new(),
            texture: String::new(),
            frame_x: 0, frame_y: 0, frame_w: 0, frame_h: 0,
            opacity: 1.0, z_index: 0,
        }
    }
    
    fn center_x(&self) -> f64 { self.x + self.width / 2.0 }
    fn center_y(&self) -> f64 { self.y + self.height / 2.0 }
    fn left(&self) -> f64 { self.x }
    fn right(&self) -> f64 { self.x + self.width }
    fn top(&self) -> f64 { self.y }
    fn bottom(&self) -> f64 { self.y + self.height }
}

pub struct CanvasModule;



#[cfg(feature = "canvas")]
impl CanvasModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            // === WINDOW ===
            "create" => Self::create(args),
            "update" => Self::update(args),
            "is_open" => Self::is_open(args),
            "clear" => Self::clear(args),
            "quit" => Self::quit(args),
            "title" => Self::set_title(args),
            
            // === DRAWING (x,y grid) ===
            "rect" => Self::rect(args),
            "fill_rect" => Self::fill_rect(args),
            "circle" => Self::circle(args),
            "fill_circle" => Self::fill_circle(args),
            "line" => Self::line(args),
            "pixel" => Self::pixel(args),
            "text" => Self::text(args),
            "triangle" => Self::triangle(args),
            "fill_triangle" => Self::fill_triangle(args),
            
            // === SPRITES - Easy game objects! ===
            "sprite" => Self::sprite_create(args),
            "set" => Self::sprite_set(args),
            "get" => Self::sprite_get(args),
            "move" => Self::sprite_move(args),
            "move_toward" => Self::sprite_move_toward(args),
            "draw" => Self::sprite_draw(args),
            "draw_all" => Self::sprite_draw_all(args),
            "delete" => Self::sprite_delete(args),
            "exists" => Self::sprite_exists(args),
            "count" => Self::sprite_count(args),
            "list" => Self::sprite_list(args),
            
            // === COLLISION ===
            "collide" => Self::collide(args),
            "collide_point" => Self::collide_point(args),
            "collide_tag" => Self::collide_tag(args),
            "collide_any" => Self::collide_any(args),
            "overlap" => Self::overlap(args),
            
            // === PHYSICS ===
            "physics" => Self::physics(args),
            "gravity" => Self::set_gravity(args),
            "velocity" => Self::set_velocity(args),
            "accelerate" => Self::accelerate(args),
            "friction" => Self::set_friction(args),
            "bounce" => Self::bounce(args),
            "wrap" => Self::wrap(args),
            "jump" => Self::jump(args),
            "platform" => Self::platform_collision(args),
            
            // === INPUT ===
            "key" => Self::key_down(args),
            "key_down" => Self::key_down(args),
            "key_pressed" => Self::key_pressed(args),
            "mouse_x" => Self::mouse_x(args),
            "mouse_y" => Self::mouse_y(args),
            "mouse" => Self::mouse_down(args),
            "mouse_down" => Self::mouse_down(args),
            "mouse_clicked" => Self::mouse_clicked(args),
            "click" => Self::mouse_clicked(args),
            
            // === CAMERA ===
            "camera" => Self::camera(args),
            "camera_follow" => Self::camera_follow(args),
            "shake" => Self::camera_shake(args),
            
            // === UTILITIES ===
            "width" => Self::get_width(args),
            "height" => Self::get_height(args),
            "rgb" => Self::rgb(args),
            "rgba" => Self::rgba(args),
            "distance" => Self::distance(args),
            "angle" => Self::angle_to(args),
            "random" => Self::random_val(args),
            "random_int" => Self::random_int(args),
            "lerp" => Self::lerp(args),
            "clamp" => Self::clamp(args),
            "delta" => Self::delta(args),
            "fps" => Self::fps(args),
            "frame" => Self::frame(args),
            "sin" => Self::sin(args),
            "cos" => Self::cos(args),
            
            // === NEW FEATURES ===
            "load_image" => Self::load_image(args),
            "draw_image" => Self::draw_image(args),
            "play_sound" => Self::play_sound(args),
            "set_z_index" => Self::set_z_index(args),
            "set_frame" => Self::set_frame(args),
            "set_texture" => Self::set_texture(args),
            "get_z_index" => Self::get_z_index(args),
            "sort_sprites" => Self::sort_sprites(args),

            
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown canvas function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }

    fn parse_color(arg: &Value) -> u32 {
        match arg {
            Value::String(s) => {
                let hex = s.trim_start_matches('#');
                u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFF)
            }
            Value::Number(n) => *n as u32,
            _ => 0xFFFFFF,
        }
    }
    
    fn get_num(args: &[Value], idx: usize, default: f64) -> f64 {
        args.get(idx).and_then(|v| match v { Value::Number(n) => Some(*n), _ => None }).unwrap_or(default)
    }
    
    fn get_str(args: &[Value], idx: usize, default: &str) -> String {
        args.get(idx).and_then(|v| match v { Value::String(s) => Some(s.clone()), _ => None }).unwrap_or_else(|| default.to_string())
    }

    // ==================== WINDOW ====================
    
    #[cfg(feature = "canvas")]
    fn create(args: &[Value]) -> MintasResult<Value> {
        let title = Self::get_str(args, 0, "Mintas Game");
        let width = Self::get_num(args, 1, 800.0) as usize;
        let height = Self::get_num(args, 2, 600.0) as usize;

        SCREEN_WIDTH.store(width as u32, Ordering::SeqCst);
        SCREEN_HEIGHT.store(height as u32, Ordering::SeqCst);

        let window = Window::new(&title, width, height, WindowOptions {
            resize: false, scale: minifb::Scale::X1, ..WindowOptions::default()
        }).map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to create window: {}", e),
            location: SourceLocation::new(0, 0),
        })?;

        unsafe {
            CANVAS_STATE = Some(CanvasState {
                buffer: vec![0; width * height],
                width, height, window: Some(window),
                keys_down: HashMap::new(), keys_pressed: HashMap::new(),
                mouse_down: [false; 3], mouse_clicked: [false; 3],
                camera_x: 0.0, camera_y: 0.0,
                sinks: Vec::new(),
                mouse_x: 0.0, mouse_y: 0.0,
            });
            SPRITES = Some(HashMap::new());
            TEXTURES = Some(HashMap::new());
            SPRITE_COUNTER.store(0, Ordering::SeqCst);
            FRAME_COUNT = 0;
            LAST_TIME = Some(std::time::Instant::now());
            
            // Init Audio
            let mut stream_lock = AUDIO_STREAM.lock().unwrap();
            if stream_lock.0.is_none() {
                if let Ok((stream, handle)) = OutputStream::try_default() {
                    *stream_lock = AudioWrapper(Some((stream, handle)));
                } else {
                    println!("[Canvas] Warning: Audio device not found");
                }
            }
        }
        GAME_RUNNING.store(true, Ordering::SeqCst);
        Ok(Value::Boolean(true))
    }



    #[cfg(feature = "canvas")]
    fn update(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            // Calculate delta time
            if let Some(last) = LAST_TIME {
                let now = std::time::Instant::now();
                DELTA_TIME = now.duration_since(last).as_secs_f64();
                LAST_TIME = Some(now);
            }
            FRAME_COUNT += 1;
            
            if let Some(ref mut state) = CANVAS_STATE {
                if let Some(ref mut window) = state.window {
                    // Clear pressed states
                    state.keys_pressed.clear();
                    let prev_mouse = state.mouse_down;
                    state.mouse_clicked = [false; 3];
                    
                    // Update keys
                    let keys = [
                        (Key::Left, "left"), (Key::Right, "right"), (Key::Up, "up"), (Key::Down, "down"),
                        (Key::Space, "space"), (Key::Enter, "enter"), (Key::Escape, "escape"),
                        (Key::LeftShift, "shift"), (Key::LeftCtrl, "ctrl"), (Key::Tab, "tab"),
                        (Key::A, "a"), (Key::B, "b"), (Key::C, "c"), (Key::D, "d"), (Key::E, "e"),
                        (Key::F, "f"), (Key::G, "g"), (Key::H, "h"), (Key::I, "i"), (Key::J, "j"),
                        (Key::K, "k"), (Key::L, "l"), (Key::M, "m"), (Key::N, "n"), (Key::O, "o"),
                        (Key::P, "p"), (Key::Q, "q"), (Key::R, "r"), (Key::S, "s"), (Key::T, "t"),
                        (Key::U, "u"), (Key::V, "v"), (Key::W, "w"), (Key::X, "x"), (Key::Y, "y"), (Key::Z, "z"),
                        (Key::Key0, "0"), (Key::Key1, "1"), (Key::Key2, "2"), (Key::Key3, "3"), (Key::Key4, "4"),
                        (Key::Key5, "5"), (Key::Key6, "6"), (Key::Key7, "7"), (Key::Key8, "8"), (Key::Key9, "9"),
                    ];
                    for (key, name) in keys.iter() {
                        let down = window.is_key_down(*key);
                        let was = state.keys_down.get(*name).copied().unwrap_or(false);
                        if down && !was { state.keys_pressed.insert(name.to_string(), true); }
                        state.keys_down.insert(name.to_string(), down);
                    }
                    
                    // Update mouse
                    // Update mouse
                    if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
                        state.mouse_x = x as f64;
                        state.mouse_y = y as f64;
                    }
                    state.mouse_down[0] = window.get_mouse_down(minifb::MouseButton::Left);
                    state.mouse_down[1] = window.get_mouse_down(minifb::MouseButton::Right);
                    state.mouse_down[2] = window.get_mouse_down(minifb::MouseButton::Middle);
                    for i in 0..3 {
                        if state.mouse_down[i] && !prev_mouse[i] { state.mouse_clicked[i] = true; }
                    }
                    
                    // Cleanup audio sinks
                    state.sinks.retain(|s| !s.empty());
                    
                    window.update_with_buffer(&state.buffer, state.width, state.height).ok();
                    return Ok(Value::Boolean(window.is_open()));
                }
            }
        }
        Ok(Value::Boolean(false))
    }



    #[cfg(feature = "canvas")]
    fn is_open(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref state) = CANVAS_STATE {
                if let Some(ref window) = state.window {
                    return Ok(Value::Boolean(window.is_open() && !window.is_key_down(Key::Escape)));
                }
            }
        }
        Ok(Value::Boolean(false))
    }


    fn quit(_args: &[Value]) -> MintasResult<Value> {
        GAME_RUNNING.store(false, Ordering::SeqCst);
        Ok(Value::Boolean(true))
    }
    
    fn set_title(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(true)) }

    #[cfg(feature = "canvas")]
    fn clear(args: &[Value]) -> MintasResult<Value> {
        let color = args.get(0).map(|v| Self::parse_color(v)).unwrap_or(0x000000);
        unsafe {
            if let Some(ref mut state) = CANVAS_STATE {
                for p in state.buffer.iter_mut() { *p = color; }
            }
        }
        Ok(Value::Boolean(true))
    }


    // ==================== DRAWING (x,y grid) ====================
    
    #[cfg(feature = "canvas")]
    fn draw_pixel(x: i32, y: i32, color: u32) {
        unsafe {
            if let Some(ref mut state) = CANVAS_STATE {
                let cx = state.camera_x as i32;
                let cy = state.camera_y as i32;
                let px = x - cx;
                let py = y - cy;
                if px >= 0 && px < state.width as i32 && py >= 0 && py < state.height as i32 {
                    state.buffer[py as usize * state.width + px as usize] = color;
                }
            }
        }
    }

    #[cfg(feature = "canvas")]
    fn fill_rect(args: &[Value]) -> MintasResult<Value> {
        let x = Self::get_num(args, 0, 0.0) as i32;
        let y = Self::get_num(args, 1, 0.0) as i32;
        let w = Self::get_num(args, 2, 50.0) as i32;
        let h = Self::get_num(args, 3, 50.0) as i32;
        let color = args.get(4).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        for dy in 0..h { for dx in 0..w { Self::draw_pixel(x + dx, y + dy, color); } }
        Ok(Value::Boolean(true))
    }


    #[cfg(feature = "canvas")]
    fn rect(args: &[Value]) -> MintasResult<Value> {
        let x = Self::get_num(args, 0, 0.0) as i32;
        let y = Self::get_num(args, 1, 0.0) as i32;
        let w = Self::get_num(args, 2, 50.0) as i32;
        let h = Self::get_num(args, 3, 50.0) as i32;
        let color = args.get(4).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        for dx in 0..w { Self::draw_pixel(x + dx, y, color); Self::draw_pixel(x + dx, y + h - 1, color); }
        for dy in 0..h { Self::draw_pixel(x, y + dy, color); Self::draw_pixel(x + w - 1, y + dy, color); }
        Ok(Value::Boolean(true))
    }


    #[cfg(feature = "canvas")]
    fn fill_circle(args: &[Value]) -> MintasResult<Value> {
        let cx = Self::get_num(args, 0, 0.0) as i32;
        let cy = Self::get_num(args, 1, 0.0) as i32;
        let r = Self::get_num(args, 2, 25.0) as i32;
        let color = args.get(3).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        for dy in -r..=r { for dx in -r..=r { if dx*dx + dy*dy <= r*r { Self::draw_pixel(cx + dx, cy + dy, color); } } }
        Ok(Value::Boolean(true))
    }


    #[cfg(feature = "canvas")]
    fn circle(args: &[Value]) -> MintasResult<Value> {
        let cx = Self::get_num(args, 0, 0.0) as i32;
        let cy = Self::get_num(args, 1, 0.0) as i32;
        let r = Self::get_num(args, 2, 25.0) as i32;
        let color = args.get(3).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        let mut x = r; let mut y = 0; let mut err = 0;
        while x >= y {
            let pts = [(cx+x,cy+y),(cx+y,cy+x),(cx-y,cy+x),(cx-x,cy+y),(cx-x,cy-y),(cx-y,cy-x),(cx+y,cy-x),(cx+x,cy-y)];
            for (px, py) in pts { Self::draw_pixel(px, py, color); }
            y += 1; err += 1 + 2*y;
            if 2*(err-x) + 1 > 0 { x -= 1; err += 1 - 2*x; }
        }
        Ok(Value::Boolean(true))
    }


    #[cfg(feature = "canvas")]
    fn line(args: &[Value]) -> MintasResult<Value> {
        let mut x1 = Self::get_num(args, 0, 0.0) as i32;
        let mut y1 = Self::get_num(args, 1, 0.0) as i32;
        let x2 = Self::get_num(args, 2, 100.0) as i32;
        let y2 = Self::get_num(args, 3, 100.0) as i32;
        let color = args.get(4).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        let dx = (x2-x1).abs(); let dy = -(y2-y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 }; let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            Self::draw_pixel(x1, y1, color);
            if x1 == x2 && y1 == y2 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x1 += sx; }
            if e2 <= dx { err += dx; y1 += sy; }
        }
        Ok(Value::Boolean(true))
    }


    #[cfg(feature = "canvas")]
    fn pixel(args: &[Value]) -> MintasResult<Value> {
        let x = Self::get_num(args, 0, 0.0) as i32;
        let y = Self::get_num(args, 1, 0.0) as i32;
        let color = args.get(2).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        Self::draw_pixel(x, y, color);
        Ok(Value::Boolean(true))
    }


    fn triangle(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(true)) }
    fn fill_triangle(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(true)) }

    // ==================== TEXT ====================
    
    #[cfg(feature = "canvas")]
    fn text(args: &[Value]) -> MintasResult<Value> {
        let txt = Self::get_str(args, 0, "");
        let x = Self::get_num(args, 1, 0.0) as i32;
        let y = Self::get_num(args, 2, 0.0) as i32;
        let color = args.get(3).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        let scale = Self::get_num(args, 4, 2.0) as i32;
        
        let mut cx = x;
        for ch in txt.chars() {
            let bitmap = Self::get_char_bitmap(ch);
            for row in 0..8 {
                for col in 0..8 {
                    if (bitmap[row] >> (7 - col)) & 1 == 1 {
                        for sy in 0..scale { for sx in 0..scale {
                            Self::draw_pixel(cx + col as i32 * scale + sx, y + row as i32 * scale + sy, color);
                        }}
                    }
                }
            }
            cx += 8 * scale + scale;
        }
        Ok(Value::Boolean(true))
    }


    #[cfg(feature = "canvas")]
    fn get_char_bitmap(ch: char) -> [u8; 8] {
        match ch.to_ascii_uppercase() {
            '0' => [0x3C,0x66,0x6E,0x76,0x66,0x66,0x3C,0x00], '1' => [0x18,0x38,0x18,0x18,0x18,0x18,0x7E,0x00],
            '2' => [0x3C,0x66,0x06,0x0C,0x18,0x30,0x7E,0x00], '3' => [0x3C,0x66,0x06,0x1C,0x06,0x66,0x3C,0x00],
            '4' => [0x0C,0x1C,0x3C,0x6C,0x7E,0x0C,0x0C,0x00], '5' => [0x7E,0x60,0x7C,0x06,0x06,0x66,0x3C,0x00],
            '6' => [0x1C,0x30,0x60,0x7C,0x66,0x66,0x3C,0x00], '7' => [0x7E,0x06,0x0C,0x18,0x30,0x30,0x30,0x00],
            '8' => [0x3C,0x66,0x66,0x3C,0x66,0x66,0x3C,0x00], '9' => [0x3C,0x66,0x66,0x3E,0x06,0x0C,0x38,0x00],
            'A' => [0x18,0x3C,0x66,0x66,0x7E,0x66,0x66,0x00], 'B' => [0x7C,0x66,0x66,0x7C,0x66,0x66,0x7C,0x00],
            'C' => [0x3C,0x66,0x60,0x60,0x60,0x66,0x3C,0x00], 'D' => [0x78,0x6C,0x66,0x66,0x66,0x6C,0x78,0x00],
            'E' => [0x7E,0x60,0x60,0x7C,0x60,0x60,0x7E,0x00], 'F' => [0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x00],
            'G' => [0x3C,0x66,0x60,0x6E,0x66,0x66,0x3E,0x00], 'H' => [0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x00],
            'I' => [0x3C,0x18,0x18,0x18,0x18,0x18,0x3C,0x00], 'J' => [0x1E,0x0C,0x0C,0x0C,0x0C,0x6C,0x38,0x00],
            'K' => [0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x00], 'L' => [0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00],
            'M' => [0x63,0x77,0x7F,0x6B,0x63,0x63,0x63,0x00], 'N' => [0x66,0x76,0x7E,0x7E,0x6E,0x66,0x66,0x00],
            'O' => [0x3C,0x66,0x66,0x66,0x66,0x66,0x3C,0x00], 'P' => [0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x00],
            'Q' => [0x3C,0x66,0x66,0x66,0x6A,0x6C,0x36,0x00], 'R' => [0x7C,0x66,0x66,0x7C,0x6C,0x66,0x66,0x00],
            'S' => [0x3C,0x66,0x60,0x3C,0x06,0x66,0x3C,0x00], 'T' => [0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x00],
            'U' => [0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00], 'V' => [0x66,0x66,0x66,0x66,0x66,0x3C,0x18,0x00],
            'W' => [0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x00], 'X' => [0x66,0x66,0x3C,0x18,0x3C,0x66,0x66,0x00],
            'Y' => [0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x00], 'Z' => [0x7E,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00],
            ' ' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], ':' => [0x00,0x18,0x18,0x00,0x18,0x18,0x00,0x00],
            '!' => [0x18,0x18,0x18,0x18,0x18,0x00,0x18,0x00], '?' => [0x3C,0x66,0x06,0x0C,0x18,0x00,0x18,0x00],
            '.' => [0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x00], ',' => [0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x30],
            '-' => [0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00], '+' => [0x00,0x18,0x18,0x7E,0x18,0x18,0x00,0x00],
            '=' => [0x00,0x00,0x7E,0x00,0x7E,0x00,0x00,0x00], '/' => [0x02,0x06,0x0C,0x18,0x30,0x60,0x40,0x00],
            _ => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        }
    }

    // ==================== SPRITES ====================
    
    // canvas.sprite("player", x, y, w, h, color) - Create sprite
    fn sprite_create(args: &[Value]) -> MintasResult<Value> {
        let id = args.get(0).and_then(|v| match v {
            Value::String(s) => Some(s.clone()), _ => None,
        }).unwrap_or_else(|| { let c = SPRITE_COUNTER.fetch_add(1, Ordering::SeqCst); format!("sprite_{}", c) });
        let x = Self::get_num(args, 1, 0.0);
        let y = Self::get_num(args, 2, 0.0);
        let w = Self::get_num(args, 3, 32.0);
        let h = Self::get_num(args, 4, 32.0);
        let color = args.get(5).map(|v| Self::parse_color(v)).unwrap_or(0xFFFFFF);
        
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                sprites.insert(id.clone(), Sprite::new(id.clone(), x, y, w, h, color));
            }
        }
        Ok(Value::String(id))
    }

    // canvas.set("player", "x", 100) - Set any property
    fn sprite_set(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let prop = Self::get_str(args, 1, "");
        let val = args.get(2).cloned().unwrap_or(Value::Number(0.0));
        
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    match prop.as_str() {
                        "x" => if let Value::Number(n) = val { s.x = n; },
                        "y" => if let Value::Number(n) = val { s.y = n; },
                        "vx" => if let Value::Number(n) = val { s.vx = n; },
                        "vy" => if let Value::Number(n) = val { s.vy = n; },
                        "ax" => if let Value::Number(n) = val { s.ax = n; },
                        "ay" => if let Value::Number(n) = val { s.ay = n; },
                        "width" | "w" => if let Value::Number(n) = val { s.width = n; },
                        "height" | "h" => if let Value::Number(n) = val { s.height = n; },
                        "color" => s.color = Self::parse_color(&val),
                        "visible" => if let Value::Boolean(b) = val { s.visible = b; },
                        "solid" => if let Value::Boolean(b) = val { s.solid = b; },
                        "gravity" => if let Value::Number(n) = val { s.gravity = n; },
                        "friction" => if let Value::Number(n) = val { s.friction = n; },
                        "tag" => if let Value::String(t) = val { s.tag = t; },
                        "health" | "hp" => if let Value::Number(n) = val { s.health = n; },
                        "speed" => if let Value::Number(n) = val { s.speed = n; },
                        "angle" => if let Value::Number(n) = val { s.angle = n; },
                        "scale" => if let Value::Number(n) = val { s.scale = n; },
                        "flip_x" => if let Value::Boolean(b) = val { s.flip_x = b; },
                        "flip_y" => if let Value::Boolean(b) = val { s.flip_y = b; },
                        "on_ground" | "grounded" => if let Value::Boolean(b) = val { s.on_ground = b; },
                        _ => { if let Value::Number(n) = val { s.data.insert(prop, n); } }
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.get("player", "x") - Get any property
    fn sprite_get(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let prop = Self::get_str(args, 1, "");
        
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let Some(s) = sprites.get(&id) {
                    return Ok(match prop.as_str() {
                        "x" => Value::Number(s.x), "y" => Value::Number(s.y),
                        "vx" => Value::Number(s.vx), "vy" => Value::Number(s.vy),
                        "ax" => Value::Number(s.ax), "ay" => Value::Number(s.ay),
                        "width" | "w" => Value::Number(s.width), "height" | "h" => Value::Number(s.height),
                        "visible" => Value::Boolean(s.visible), "solid" => Value::Boolean(s.solid),
                        "gravity" => Value::Number(s.gravity), "friction" => Value::Number(s.friction),
                        "on_ground" | "grounded" => Value::Boolean(s.on_ground),
                        "tag" => Value::String(s.tag.clone()),
                        "health" | "hp" => Value::Number(s.health), "speed" => Value::Number(s.speed),
                        "angle" => Value::Number(s.angle), "scale" => Value::Number(s.scale),
                        "flip_x" => Value::Boolean(s.flip_x), "flip_y" => Value::Boolean(s.flip_y),
                        "cx" | "center_x" => Value::Number(s.center_x()),
                        "cy" | "center_y" => Value::Number(s.center_y()),
                        "left" => Value::Number(s.left()), "right" => Value::Number(s.right()),
                        "top" => Value::Number(s.top()), "bottom" => Value::Number(s.bottom()),
                        _ => s.data.get(&prop).map(|n| Value::Number(*n)).unwrap_or(Value::Empty),
                    });
                }
            }
        }
        Ok(Value::Empty)
    }

    // canvas.move("player", dx, dy) - Move by delta
    fn sprite_move(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let dx = Self::get_num(args, 1, 0.0);
        let dy = Self::get_num(args, 2, 0.0);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) { s.x += dx; s.y += dy; }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.move_toward("player", target_x, target_y, speed) - Move toward point
    fn sprite_move_toward(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let tx = Self::get_num(args, 1, 0.0);
        let ty = Self::get_num(args, 2, 0.0);
        let spd = Self::get_num(args, 3, 5.0);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    let dx = tx - s.center_x();
                    let dy = ty - s.center_y();
                    let dist = (dx*dx + dy*dy).sqrt();
                    if dist > 0.0 {
                        s.x += (dx / dist) * spd;
                        s.y += (dy / dist) * spd;
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    fn sprite_delete(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        unsafe { if let Some(ref mut sprites) = SPRITES { sprites.remove(&id); } }
        Ok(Value::Boolean(true))
    }

    fn sprite_exists(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        unsafe {
            if let Some(ref sprites) = SPRITES { return Ok(Value::Boolean(sprites.contains_key(&id))); }
        }
        Ok(Value::Boolean(false))
    }

    fn sprite_count(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref sprites) = SPRITES { return Ok(Value::Number(sprites.len() as f64)); }
        }
        Ok(Value::Number(0.0))
    }

    fn sprite_list(_args: &[Value]) -> MintasResult<Value> {
        let mut list = Vec::new();
        unsafe {
            if let Some(ref sprites) = SPRITES {
                for id in sprites.keys() { list.push(Value::String(id.clone())); }
            }
        }
        Ok(Value::Array(list))
    }

    // canvas.draw("player") - Draw one sprite
    #[cfg(feature = "canvas")]
    fn sprite_draw(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let Some(s) = sprites.get(&id) {
                    if s.visible {
                        if !s.texture.is_empty() {
                            if let Some(ref textures) = TEXTURES {
                                if let Some(tex) = textures.get(&s.texture) {
                                    Self::blit_texture(tex, s.x as i32, s.y as i32, 
                                        s.width as i32, s.height as i32, 
                                        s.angle, s.scale, 
                                        s.frame_x, s.frame_y, s.frame_w, s.frame_h);
                                    return Ok(Value::Boolean(true));
                                }
                            }
                        }
                        let x = s.x as i32; let y = s.y as i32;
                        let w = (s.width * s.scale) as i32; let h = (s.height * s.scale) as i32;
                        
                        if let Some(ref mut state) = CANVAS_STATE {
                            let cx = state.camera_x as i32;
                            let cy = state.camera_y as i32;
                            for dy in 0..h { for dx in 0..w { 
                                let px = x + dx - cx; let py = y + dy - cy;
                                if px >= 0 && px < state.width as i32 && py >= 0 && py < state.height as i32 {
                                    state.buffer[py as usize * state.width + px as usize] = s.color;
                                }
                            }}
                        }
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }


    // canvas.draw_all() - Draw ALL sprites
    
    fn sprite_draw_all(_args: &[Value]) -> MintasResult<Value> {
        unsafe {
            if let Some(ref sprites) = SPRITES {
                let mut sorted: Vec<&Sprite> = sprites.values().collect();
                sorted.sort_by_key(|s| s.z_index);
                
                for s in sorted {
                    if s.visible {
                        if !s.texture.is_empty() {
                            if let Some(ref textures) = TEXTURES {
                                if let Some(tex) = textures.get(&s.texture) {
                                    Self::blit_texture(tex, s.x as i32, s.y as i32, 
                                        s.width as i32, s.height as i32, 
                                        s.angle, s.scale, 
                                        s.frame_x, s.frame_y, s.frame_w, s.frame_h);
                                    continue;
                                }
                            }
                        }
                        // Fallback to rect
                        let x = s.x as i32; let y = s.y as i32;
                        let w = (s.width * s.scale) as i32; let h = (s.height * s.scale) as i32;
                        
                        // Simple rect drawing (no rotation for primitives yet)
                        if let Some(ref mut state) = CANVAS_STATE {
                            let cx = state.camera_x as i32;
                            let cy = state.camera_y as i32;
                            for dy in 0..h { for dx in 0..w { 
                                let px = x + dx - cx; let py = y + dy - cy;
                                if px >= 0 && px < state.width as i32 && py >= 0 && py < state.height as i32 {
                                    state.buffer[py as usize * state.width + px as usize] = s.color;
                                }
                            }}
                        }
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }


    // ==================== COLLISION ====================
    
    // canvas.collide("player", "enemy") - AABB collision
    fn collide(args: &[Value]) -> MintasResult<Value> {
        let id1 = Self::get_str(args, 0, "");
        let id2 = Self::get_str(args, 1, "");
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let (Some(a), Some(b)) = (sprites.get(&id1), sprites.get(&id2)) {
                    let hit = a.x < b.x + b.width && a.x + a.width > b.x &&
                              a.y < b.y + b.height && a.y + a.height > b.y;
                    return Ok(Value::Boolean(hit));
                }
            }
        }
        Ok(Value::Boolean(false))
    }

    // canvas.collide_point("player", x, y)
    fn collide_point(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let px = Self::get_num(args, 1, 0.0);
        let py = Self::get_num(args, 2, 0.0);
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let Some(s) = sprites.get(&id) {
                    let hit = px >= s.x && px <= s.x + s.width && py >= s.y && py <= s.y + s.height;
                    return Ok(Value::Boolean(hit));
                }
            }
        }
        Ok(Value::Boolean(false))
    }

    // canvas.collide_tag("player", "enemy") - Returns ID of collided sprite
    fn collide_tag(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let tag = Self::get_str(args, 1, "");
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let Some(a) = sprites.get(&id) {
                    for b in sprites.values() {
                        if b.tag == tag && b.id != id {
                            let hit = a.x < b.x + b.width && a.x + a.width > b.x &&
                                      a.y < b.y + b.height && a.y + a.height > b.y;
                            if hit { return Ok(Value::String(b.id.clone())); }
                        }
                    }
                }
            }
        }
        Ok(Value::Boolean(false))
    }

    // canvas.collide_any("player") - Check collision with any sprite
    fn collide_any(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let Some(a) = sprites.get(&id) {
                    for b in sprites.values() {
                        if b.id != id && b.solid {
                            let hit = a.x < b.x + b.width && a.x + a.width > b.x &&
                                      a.y < b.y + b.height && a.y + a.height > b.y;
                            if hit { return Ok(Value::String(b.id.clone())); }
                        }
                    }
                }
            }
        }
        Ok(Value::Boolean(false))
    }

    // canvas.overlap("player", "coin") - Same as collide but returns overlap amount
    fn overlap(args: &[Value]) -> MintasResult<Value> {
        let id1 = Self::get_str(args, 0, "");
        let id2 = Self::get_str(args, 1, "");
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let (Some(a), Some(b)) = (sprites.get(&id1), sprites.get(&id2)) {
                    let ox = (a.right().min(b.right()) - a.left().max(b.left())).max(0.0);
                    let oy = (a.bottom().min(b.bottom()) - a.top().max(b.top())).max(0.0);
                    if ox > 0.0 && oy > 0.0 {
                        let mut result = HashMap::new();
                        result.insert("x".to_string(), Value::Number(ox));
                        result.insert("y".to_string(), Value::Number(oy));
                        return Ok(Value::Table(result));
                    }
                }
            }
        }
        Ok(Value::Boolean(false))
    }

    // ==================== PHYSICS ====================
    
    // canvas.physics("player", ground_y) - Apply velocity, gravity, friction
    fn physics(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let ground = args.get(1).and_then(|v| match v { Value::Number(n) => Some(*n), _ => None });
        
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    // Apply acceleration
                    s.vx += s.ax;
                    s.vy += s.ay;
                    // Apply gravity
                    s.vy += s.gravity;
                    // Apply friction
                    s.vx *= s.friction;
                    // Apply velocity
                    s.x += s.vx;
                    s.y += s.vy;
                    // Ground collision
                    s.on_ground = false;
                    if let Some(g) = ground {
                        if s.y + s.height >= g {
                            s.y = g - s.height;
                            s.vy = 0.0;
                            s.on_ground = true;
                        }
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.gravity("player", 0.5)
    fn set_gravity(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let g = Self::get_num(args, 1, 0.5);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) { s.gravity = g; }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.velocity("player", vx, vy)
    fn set_velocity(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let vx = Self::get_num(args, 1, 0.0);
        let vy = Self::get_num(args, 2, 0.0);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) { s.vx = vx; s.vy = vy; }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.accelerate("player", ax, ay)
    fn accelerate(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let ax = Self::get_num(args, 1, 0.0);
        let ay = Self::get_num(args, 2, 0.0);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) { s.vx += ax; s.vy += ay; }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.friction("player", 0.9)
    fn set_friction(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let f = Self::get_num(args, 1, 0.9);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) { s.friction = f; }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.bounce("player", min_x, max_x, min_y, max_y)
    fn bounce(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let min_x = Self::get_num(args, 1, 0.0);
        let max_x = Self::get_num(args, 2, SCREEN_WIDTH.load(Ordering::SeqCst) as f64);
        let min_y = Self::get_num(args, 3, 0.0);
        let max_y = Self::get_num(args, 4, SCREEN_HEIGHT.load(Ordering::SeqCst) as f64);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    if s.x < min_x { s.x = min_x; s.vx = -s.vx; }
                    if s.x + s.width > max_x { s.x = max_x - s.width; s.vx = -s.vx; }
                    if s.y < min_y { s.y = min_y; s.vy = -s.vy; }
                    if s.y + s.height > max_y { s.y = max_y - s.height; s.vy = -s.vy; }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.wrap("player") - Wrap around screen edges
    fn wrap(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let sw = SCREEN_WIDTH.load(Ordering::SeqCst) as f64;
        let sh = SCREEN_HEIGHT.load(Ordering::SeqCst) as f64;
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    if s.x + s.width < 0.0 { s.x = sw; }
                    if s.x > sw { s.x = -s.width; }
                    if s.y + s.height < 0.0 { s.y = sh; }
                    if s.y > sh { s.y = -s.height; }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    // canvas.jump("player", force) - Jump if on ground
    fn jump(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let force = Self::get_num(args, 1, 12.0);
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    if s.on_ground {
                        s.vy = -force;
                        s.on_ground = false;
                        return Ok(Value::Boolean(true));
                    }
                }
            }
        }
        Ok(Value::Boolean(false))
    }

    // canvas.platform("player", "platform") - Platform collision
    fn platform_collision(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let plat_id = Self::get_str(args, 1, "");
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                let plat = sprites.get(&plat_id).cloned();
                if let (Some(ref mut s), Some(p)) = (sprites.get_mut(&id), plat) {
                    // Only collide from above
                    if s.vy >= 0.0 && s.bottom() >= p.top() && s.bottom() <= p.top() + 15.0 &&
                       s.right() > p.left() && s.left() < p.right() {
                        s.y = p.top() - s.height;
                        s.vy = 0.0;
                        s.on_ground = true;
                        return Ok(Value::Boolean(true));
                    }
                }
            }
        }
        Ok(Value::Boolean(false))
    }

    // ==================== INPUT ====================
    
    #[cfg(feature = "canvas")]
    fn key_down(args: &[Value]) -> MintasResult<Value> {
        let key = Self::get_str(args, 0, "").to_lowercase();
        unsafe {
            if let Some(ref state) = CANVAS_STATE {
                return Ok(Value::Boolean(state.keys_down.get(&key).copied().unwrap_or(false)));
            }
        }
        Ok(Value::Boolean(false))
    }
    #[cfg(not(feature = "canvas"))]
    fn key_down(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(false)) }

    #[cfg(feature = "canvas")]
    fn key_pressed(args: &[Value]) -> MintasResult<Value> {
        let key = Self::get_str(args, 0, "").to_lowercase();
        unsafe {
            if let Some(ref state) = CANVAS_STATE {
                return Ok(Value::Boolean(state.keys_pressed.get(&key).copied().unwrap_or(false)));
            }
        }
        Ok(Value::Boolean(false))
    }
    #[cfg(not(feature = "canvas"))]
    fn key_pressed(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(false)) }

    #[cfg(feature = "canvas")]
    fn mouse_x(_args: &[Value]) -> MintasResult<Value> {
        unsafe { if let Some(ref state) = CANVAS_STATE { return Ok(Value::Number(state.mouse_x)); } }
        Ok(Value::Number(0.0))
    }
    #[cfg(not(feature = "canvas"))]
    fn mouse_x(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Number(0.0)) }

    #[cfg(feature = "canvas")]
    fn mouse_y(_args: &[Value]) -> MintasResult<Value> {
        unsafe { if let Some(ref state) = CANVAS_STATE { return Ok(Value::Number(state.mouse_y)); } }
        Ok(Value::Number(0.0))
    }
    #[cfg(not(feature = "canvas"))]
    fn mouse_y(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Number(0.0)) }

    #[cfg(feature = "canvas")]
    fn mouse_down(args: &[Value]) -> MintasResult<Value> {
        let btn = Self::get_num(args, 0, 0.0) as usize;
        unsafe {
            if let Some(ref state) = CANVAS_STATE {
                if btn < 3 { return Ok(Value::Boolean(state.mouse_down[btn])); }
            }
        }
        Ok(Value::Boolean(false))
    }
    #[cfg(not(feature = "canvas"))]
    fn mouse_down(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(false)) }

    #[cfg(feature = "canvas")]
    fn mouse_clicked(args: &[Value]) -> MintasResult<Value> {
        let btn = Self::get_num(args, 0, 0.0) as usize;
        unsafe {
            if let Some(ref state) = CANVAS_STATE {
                if btn < 3 { return Ok(Value::Boolean(state.mouse_clicked[btn])); }
            }
        }
        Ok(Value::Boolean(false))
    }
    #[cfg(not(feature = "canvas"))]
    fn mouse_clicked(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(false)) }

    // ==================== CAMERA ====================
    
    fn camera(args: &[Value]) -> MintasResult<Value> {
        let x = Self::get_num(args, 0, 0.0);
        let y = Self::get_num(args, 1, 0.0);
        unsafe {
            if let Some(ref mut state) = CANVAS_STATE {
                state.camera_x = x;
                state.camera_y = y;
            }
        }
        Ok(Value::Boolean(true))
    }

    fn camera_follow(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let smooth = Self::get_num(args, 1, 1.0);
        unsafe {
            if let Some(ref sprites) = SPRITES {
                if let Some(s) = sprites.get(&id) {
                    let sw = SCREEN_WIDTH.load(Ordering::SeqCst) as f64;
                    let sh = SCREEN_HEIGHT.load(Ordering::SeqCst) as f64;
                    let tx = s.center_x() - sw / 2.0;
                    let ty = s.center_y() - sh / 2.0;
                    if let Some(ref mut state) = CANVAS_STATE {
                        state.camera_x += (tx - state.camera_x) * smooth;
                        state.camera_y += (ty - state.camera_y) * smooth;
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    fn camera_shake(args: &[Value]) -> MintasResult<Value> {
        let intensity = Self::get_num(args, 0, 5.0);
        unsafe {
            if let Some(ref mut state) = CANVAS_STATE {
                let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as f64;
                let rx = ((seed * 1103515245.0 + 12345.0) % 2147483648.0) / 2147483648.0 - 0.5;
                let ry = ((seed * 1103515245.0 + 54321.0) % 2147483648.0) / 2147483648.0 - 0.5;
                state.camera_x += rx * intensity * 2.0;
                state.camera_y += ry * intensity * 2.0;
            }
        }
        Ok(Value::Boolean(true))
    }

    // ==================== UTILITIES ====================
    
    fn get_width(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Number(SCREEN_WIDTH.load(Ordering::SeqCst) as f64))
    }

    fn get_height(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Number(SCREEN_HEIGHT.load(Ordering::SeqCst) as f64))
    }

    fn rgb(args: &[Value]) -> MintasResult<Value> {
        let r = Self::get_num(args, 0, 255.0) as u32;
        let g = Self::get_num(args, 1, 255.0) as u32;
        let b = Self::get_num(args, 2, 255.0) as u32;
        Ok(Value::Number(((r << 16) | (g << 8) | b) as f64))
    }

    fn rgba(args: &[Value]) -> MintasResult<Value> {
        let r = Self::get_num(args, 0, 255.0) as u32;
        let g = Self::get_num(args, 1, 255.0) as u32;
        let b = Self::get_num(args, 2, 255.0) as u32;
        Ok(Value::Number(((r << 16) | (g << 8) | b) as f64))
    }

    fn distance(args: &[Value]) -> MintasResult<Value> {
        let x1 = Self::get_num(args, 0, 0.0);
        let y1 = Self::get_num(args, 1, 0.0);
        let x2 = Self::get_num(args, 2, 0.0);
        let y2 = Self::get_num(args, 3, 0.0);
        let dx = x2 - x1; let dy = y2 - y1;
        Ok(Value::Number((dx*dx + dy*dy).sqrt()))
    }

    fn angle_to(args: &[Value]) -> MintasResult<Value> {
        let x1 = Self::get_num(args, 0, 0.0);
        let y1 = Self::get_num(args, 1, 0.0);
        let x2 = Self::get_num(args, 2, 0.0);
        let y2 = Self::get_num(args, 3, 0.0);
        Ok(Value::Number((y2 - y1).atan2(x2 - x1).to_degrees()))
    }

    fn random_val(args: &[Value]) -> MintasResult<Value> {
        let min = Self::get_num(args, 0, 0.0);
        let max = Self::get_num(args, 1, 1.0);
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as f64;
        let r = ((seed * 1103515245.0 + 12345.0) % 2147483648.0) / 2147483648.0;
        Ok(Value::Number(min + r * (max - min)))
    }

    fn random_int(args: &[Value]) -> MintasResult<Value> {
        let min = Self::get_num(args, 0, 0.0) as i64;
        let max = Self::get_num(args, 1, 100.0) as i64;
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as i64;
        let r = ((seed * 1103515245 + 12345) % 2147483648).abs();
        Ok(Value::Number((min + r % (max - min + 1)) as f64))
    }

    fn lerp(args: &[Value]) -> MintasResult<Value> {
        let a = Self::get_num(args, 0, 0.0);
        let b = Self::get_num(args, 1, 1.0);
        let t = Self::get_num(args, 2, 0.5);
        Ok(Value::Number(a + (b - a) * t))
    }

    fn clamp(args: &[Value]) -> MintasResult<Value> {
        let val = Self::get_num(args, 0, 0.0);
        let min = Self::get_num(args, 1, 0.0);
        let max = Self::get_num(args, 2, 1.0);
        Ok(Value::Number(val.clamp(min, max)))
    }

    fn delta(_args: &[Value]) -> MintasResult<Value> {
        unsafe { Ok(Value::Number(DELTA_TIME)) }
    }

    fn fps(_args: &[Value]) -> MintasResult<Value> {
        unsafe { Ok(Value::Number(if DELTA_TIME > 0.0 { 1.0 / DELTA_TIME } else { 60.0 })) }
    }

    fn frame(_args: &[Value]) -> MintasResult<Value> {
        unsafe { Ok(Value::Number(FRAME_COUNT as f64)) }
    }

    fn sin(args: &[Value]) -> MintasResult<Value> {
        let deg = Self::get_num(args, 0, 0.0);
        Ok(Value::Number(deg.to_radians().sin()))
    }

    fn cos(args: &[Value]) -> MintasResult<Value> {
        let deg = Self::get_num(args, 0, 0.0);
        Ok(Value::Number(deg.to_radians().cos()))
    }

    // ==================== EXTENDED FEATURES ====================

    #[cfg(feature = "canvas")]
    fn load_image(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let path = Self::get_str(args, 1, "");
        
        let img = image::open(&path).map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to load image {}: {}", path, e),
            location: SourceLocation::new(0, 0),
        })?;
        let img = img.to_rgba8();
        let width = img.width();
        let height = img.height();
        
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for pixel in img.pixels() {
            let [r, g, b, a] = pixel.0;
            pixels.push((a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32));
        }
        
        unsafe {
            if let Some(ref mut textures) = TEXTURES {
                textures.insert(id.clone(), Texture { width, height, pixels });
            }
        }
        Ok(Value::Boolean(true))
    }

    #[cfg(feature = "canvas")]
    fn play_sound(args: &[Value]) -> MintasResult<Value> {
        let path = Self::get_str(args, 0, "");
        if let Ok(file) = std::fs::File::open(&path) {
            let source = Decoder::new(std::io::BufReader::new(file)).ok();
            if let Some(source) = source {
                let stream_lock = AUDIO_STREAM.lock().unwrap();
                if let Some((_, handle)) = &stream_lock.0 {
                    if let Ok(sink) = Sink::try_new(handle) {
                        sink.append(source);
                        unsafe {
                            if let Some(ref mut state) = CANVAS_STATE {
                                state.sinks.push(sink);
                            }
                        }
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    #[cfg(feature = "canvas")]
    fn set_texture(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let tex = Self::get_str(args, 1, "");
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    s.texture = tex;
                    // Auto-resize sprite to texture if 0 dims
                    if let Some(ref textures) = TEXTURES {
                        if let Some(t) = textures.get(&s.texture) {
                             if s.width == 0.0 { s.width = t.width as f64; }
                             if s.height == 0.0 { s.height = t.height as f64; }
                             s.frame_w = t.width as i32;
                             s.frame_h = t.height as i32;
                        }
                    }
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    #[cfg(feature = "canvas")]
    fn set_frame(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let fx = Self::get_num(args, 1, 0.0) as i32;
        let fy = Self::get_num(args, 2, 0.0) as i32;
        let fw = Self::get_num(args, 3, 32.0) as i32;
        let fh = Self::get_num(args, 4, 32.0) as i32;
        unsafe {
            if let Some(ref mut sprites) = SPRITES {
                if let Some(s) = sprites.get_mut(&id) {
                    s.frame_x = fx; s.frame_y = fy; s.frame_w = fw; s.frame_h = fh;
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    #[cfg(feature = "canvas")]
    fn set_z_index(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let z = Self::get_num(args, 1, 0.0) as i32;
        unsafe { if let Some(ref mut sprites) = SPRITES { if let Some(s) = sprites.get_mut(&id) { s.z_index = z; } } }
        Ok(Value::Boolean(true))
    }

    #[cfg(feature = "canvas")]
    fn get_z_index(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        unsafe { if let Some(ref sprites) = SPRITES { if let Some(s) = sprites.get(&id) { return Ok(Value::Number(s.z_index as f64)); } } }
        Ok(Value::Number(0.0))
    }

    #[cfg(feature = "canvas")]
    fn sort_sprites(_args: &[Value]) -> MintasResult<Value> {
        // Sorting in HashMap is impossible, but draw_all can sort keys
        Ok(Value::Boolean(true))
    }

    #[cfg(feature = "canvas")]
    fn draw_image(args: &[Value]) -> MintasResult<Value> {
        let id = Self::get_str(args, 0, "");
        let x = Self::get_num(args, 1, 0.0) as i32;
        let y = Self::get_num(args, 2, 0.0) as i32;
        unsafe {
            if let Some(ref textures) = TEXTURES {
                if let Some(tex) = textures.get(&id) {
                    Self::blit_texture(tex, x, y, tex.width as i32, tex.height as i32, 0.0, 1.0, 0, 0, tex.width as i32, tex.height as i32);
                }
            }
        }
        Ok(Value::Boolean(true))
    }

    #[cfg(feature = "canvas")]
    fn blit_texture(tex: &Texture, x: i32, y: i32, target_w: i32, target_h: i32, angle: f64, scale: f64, sx: i32, sy: i32, sw: i32, sh: i32) {
        unsafe {
            if let Some(ref mut state) = CANVAS_STATE {
                let cx = state.camera_x as i32;
                let cy = state.camera_y as i32;
                let screen_w = state.width as i32;
                let screen_h = state.height as i32;

                // No rotation optimization
                if angle.abs() < 0.1 {
                     let start_x = x - cx;
                     let start_y = y - cy;
                     let w = (target_w as f64 * scale) as i32;
                     let h = (target_h as f64 * scale) as i32;
                     
                     // Clipping
                     let mut render_x = start_x;
                     let mut render_y = start_y;
                     let mut render_w = w;
                     let mut render_h = h;
                     let mut src_start_x = sx;
                     let mut src_start_y = sy;

                     if render_x < 0 {
                         let diff = -render_x;
                         render_w -= diff;
                         src_start_x += (diff as f64 / scale) as i32;
                         render_x = 0;
                     }
                     if render_y < 0 {
                         let diff = -render_y;
                         render_h -= diff;
                         src_start_y += (diff as f64 / scale) as i32;
                         render_y = 0;
                     }
                     if render_x + render_w > screen_w { render_w = screen_w - render_x; }
                     if render_y + render_h > screen_h { render_h = screen_h - render_y; }

                     if render_w <= 0 || render_h <= 0 { return; }

                    // Draw loop
                    let scale_inv = 1.0 / scale;
                    for dy in 0..render_h {
                        let tex_y = src_start_y + (dy as f64 * scale_inv) as i32;
                        if tex_y >= sy + sh { break; }
                        let dest_row_start = (render_y + dy) as usize * state.width;
                        let src_row_start = tex_y as usize * tex.width as usize;
                        
                        for dx in 0..render_w {
                            let tex_x = src_start_x + (dx as f64 * scale_inv) as i32;
                            if tex_x >= sx + sw { break; }
                            
                            let color = tex.pixels[src_row_start + tex_x as usize];
                            if (color >> 24) > 0 { // Check alpha
                                state.buffer[dest_row_start + (render_x + dx) as usize] = color;
                            }
                        }
                    }
                } else {
                    // Rotated rendering (Nearest Neighbor)
                    let rad = angle.to_radians();
                    let cos_a = rad.cos();
                    let sin_a = rad.sin();
                    let cx_center = x as f64 + (target_w as f64 * scale) / 2.0;
                    let cy_center = y as f64 + (target_h as f64 * scale) / 2.0;

                    // Compute bounding box
                    let w = target_w as f64 * scale;
                    let h = target_h as f64 * scale;
                    let corners = [
                        (-w/2.0, -h/2.0), (w/2.0, -h/2.0),
                        (w/2.0, h/2.0), (-w/2.0, h/2.0)
                    ];
                    
                    let mut min_x = 10000.0; let mut max_x = -10000.0;
                    let mut min_y = 10000.0; let mut max_y = -10000.0;
                    
                    for (bx, by) in corners {
                        let rx = bx * cos_a - by * sin_a + cx_center;
                        let ry = bx * sin_a + by * cos_a + cy_center;
                        if rx < min_x { min_x = rx; }
                        if rx > max_x { max_x = rx; }
                        if ry < min_y { min_y = ry; }
                        if ry > max_y { max_y = ry; }
                    }
                    
                    let start_x = (min_x - state.camera_x) as i32;
                    let start_y = (min_y - state.camera_y) as i32;
                    let end_x = (max_x - state.camera_x) as i32;
                    let end_y = (max_y - state.camera_y) as i32;
                    
                    for dy in start_y..end_y {
                        if dy < 0 || dy >= screen_h { continue; }
                        for dx in start_x..end_x {
                            if dx < 0 || dx >= screen_w { continue; }
                            
                            // Inverse transform
                            let screen_x = dx as f64 + state.camera_x;
                            let screen_y = dy as f64 + state.camera_y;
                            let local_x = screen_x - cx_center;
                            let local_y = screen_y - cy_center;
                            
                            let rot_x = local_x * cos_a + local_y * sin_a;
                            let rot_y = -local_x * sin_a + local_y * cos_a;
                            
                            let tex_x = (rot_x / scale) + (target_w as f64 / 2.0);
                            let tex_y = (rot_y / scale) + (target_h as f64 / 2.0);
                            
                            if tex_x >= 0.0 && tex_x < target_w as f64 && tex_y >= 0.0 && tex_y < target_h as f64 {
                                // Add clipping to source rect
                                let final_tx = sx + tex_x as i32;
                                let final_ty = sy + tex_y as i32;
                                if final_tx >= sx && final_tx < sx + sw && final_ty >= sy && final_ty < sy + sh {
                                     let color = tex.pixels[final_ty as usize * tex.width as usize + final_tx as usize];
                                     if (color >> 24) > 0 {
                                          state.buffer[dy as usize * state.width + dx as usize] = color;
                                     }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

