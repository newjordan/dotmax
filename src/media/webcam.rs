//! Webcam capture and live display support.
//!
//! This module provides [`WebcamPlayer`] for live webcam capture, implementing
//! the [`MediaPlayer`] trait for integration with the universal media system.
//!
//! # Requirements
//!
//! This module requires:
//! 1. The `video` feature flag enabled in Cargo.toml
//! 2. FFmpeg libraries installed on the system
//! 3. A connected webcam/capture device
//!
//! # Platform Support
//!
//! Webcam capture uses platform-specific device APIs via FFmpeg:
//! - **Linux**: V4L2 (Video4Linux2) - devices like `/dev/video0`
//! - **macOS**: AVFoundation - devices by index or name
//! - **Windows**: DirectShow - devices by name
//!
//! # Examples
//!
//! ## Basic Webcam Display
//!
//! ```no_run
//! use dotmax::media::{WebcamPlayer, MediaPlayer};
//!
//! let mut player = WebcamPlayer::new()?;
//! while let Some(result) = player.next_frame() {
//!     let (grid, _delay) = result?;
//!     // Render grid to terminal
//! }
//! # Ok::<(), dotmax::DotmaxError>(())
//! ```
//!
//! ## List Available Cameras
//!
//! ```no_run
//! use dotmax::media::list_webcams;
//!
//! let cameras = list_webcams();
//! for cam in cameras {
//!     println!("{}: {} ({})", cam.id, cam.name, cam.description);
//! }
//! ```
//!
//! ## Using Builder Pattern
//!
//! ```no_run
//! use dotmax::media::WebcamPlayer;
//! use dotmax::image::DitheringMethod;
//!
//! let player = WebcamPlayer::builder()
//!     .device(0)  // First camera
//!     .resolution(1280, 720)
//!     .fps(30)
//!     .dithering(DitheringMethod::Bayer)
//!     .build()?;
//! # Ok::<(), dotmax::DotmaxError>(())
//! ```
//!
//! # Architecture
//!
//! `WebcamPlayer` uses FFmpeg (via `ffmpeg-next` crate) for device capture:
//!
//! 1. **Device Opening**: Platform-specific input format (v4l2/avfoundation/dshow)
//! 2. **Decoding**: `codec::decoder::Video` decodes raw frames
//! 3. **Scaling**: `software::scaling::Context` converts to RGB24
//! 4. **Rendering**: `ImageRenderer` converts RGB data to `BrailleGrid`
//!
//! # Thread Safety
//!
//! `WebcamPlayer` is `Send` but not `Sync`. It can be moved between threads
//! but should not be accessed from multiple threads simultaneously.

use std::time::Duration;

use crate::image::{ColorMode, DitheringMethod};
use crate::{BrailleGrid, DotmaxError, Result};

use super::MediaPlayer;

extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::Pixel;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context as ScalingContext, flag::Flags};
use ffmpeg::util::frame::video::Video as VideoFrame;

// ============================================================================
// WebcamDevice (AC: #2)
// ============================================================================

/// Information about an available webcam device.
///
/// This struct contains identifying information for a webcam that can be
/// used to open it with [`WebcamPlayer::from_device()`].
///
/// # Examples
///
/// ```no_run
/// use dotmax::media::list_webcams;
///
/// for device in list_webcams() {
///     println!("Camera: {} ({})", device.name, device.id);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebcamDevice {
    /// Platform-specific device identifier.
    ///
    /// - Linux: Device path (e.g., `/dev/video0`)
    /// - macOS: Device index as string (e.g., `"0"`)
    /// - Windows: Device name (e.g., `"Integrated Camera"`)
    pub id: String,

    /// Human-readable device name.
    pub name: String,

    /// Additional description or capabilities.
    pub description: String,
}

impl WebcamDevice {
    /// Creates a new `WebcamDevice` with the given identifiers.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
        }
    }
}

// ============================================================================
// WebcamDeviceId (AC: #4)
// ============================================================================

/// Device identifier for webcam selection.
///
/// This enum allows flexible device selection by:
/// - Index (0, 1, 2, ...)
/// - Path (Linux: `/dev/video0`)
/// - Name (Windows/macOS: `"FaceTime HD Camera"`)
#[derive(Debug, Clone, Default)]
pub enum WebcamDeviceId {
    /// Default system webcam.
    #[default]
    Default,
    /// Camera by index (0 = first camera).
    Index(usize),
    /// Camera by device path (Linux) or name (Windows/macOS).
    Path(String),
}

impl From<usize> for WebcamDeviceId {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl From<&str> for WebcamDeviceId {
    fn from(path: &str) -> Self {
        Self::Path(path.to_string())
    }
}

impl From<String> for WebcamDeviceId {
    fn from(path: String) -> Self {
        Self::Path(path)
    }
}

impl std::fmt::Display for WebcamDeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Index(i) => write!(f, "index:{i}"),
            Self::Path(p) => write!(f, "{p}"),
        }
    }
}

// ============================================================================
// Device Enumeration (AC: #2)
// ============================================================================

/// Lists available webcam devices on the system.
///
/// This function queries the operating system for connected camera devices.
/// The returned list may be empty if no cameras are detected.
///
/// # Platform Behavior
///
/// - **Linux**: Enumerates `/dev/video*` devices
/// - **macOS**: Queries AVFoundation for capture devices
/// - **Windows**: Queries DirectShow for video input devices
///
/// # Returns
///
/// A vector of [`WebcamDevice`] structs, one per detected camera.
/// Returns an empty vector if no cameras are found (not an error).
///
/// # Examples
///
/// ```no_run
/// use dotmax::media::list_webcams;
///
/// let cameras = list_webcams();
/// if cameras.is_empty() {
///     println!("No cameras detected");
/// } else {
///     for (i, cam) in cameras.iter().enumerate() {
///         println!("[{}] {}: {}", i, cam.name, cam.description);
///     }
/// }
/// ```
#[must_use]
pub fn list_webcams() -> Vec<WebcamDevice> {
    #[cfg(target_os = "linux")]
    {
        list_webcams_linux()
    }
    #[cfg(target_os = "macos")]
    {
        list_webcams_macos()
    }
    #[cfg(target_os = "windows")]
    {
        list_webcams_windows()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        vec![]
    }
}

/// Linux: Enumerate V4L2 devices.
#[cfg(target_os = "linux")]
fn list_webcams_linux() -> Vec<WebcamDevice> {
    use std::fs;

    let mut devices = Vec::new();

    // Enumerate /dev/video* devices
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("video") {
                    let device_path = path.to_string_lossy().to_string();

                    // Try to get device name from sysfs
                    let device_name = get_v4l2_device_name(&device_path)
                        .unwrap_or_else(|| name.to_string());

                    devices.push(WebcamDevice {
                        id: device_path.clone(),
                        name: device_name,
                        description: format!("V4L2 device at {device_path}"),
                    });
                }
            }
        }
    }

    // Sort by device path for consistent ordering
    devices.sort_by(|a, b| a.id.cmp(&b.id));
    devices
}

/// Get V4L2 device name from sysfs.
#[cfg(target_os = "linux")]
fn get_v4l2_device_name(device_path: &str) -> Option<String> {
    use std::fs;

    // Extract video number from path (e.g., "/dev/video0" -> "video0")
    let device_name = device_path.strip_prefix("/dev/")?;

    // Try to read name from sysfs
    let sysfs_path = format!("/sys/class/video4linux/{device_name}/name");
    fs::read_to_string(sysfs_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// macOS: Query AVFoundation devices using FFmpeg.
#[cfg(target_os = "macos")]
fn list_webcams_macos() -> Vec<WebcamDevice> {
    // Use FFmpeg to enumerate AVFoundation devices
    // Run: ffmpeg -f avfoundation -list_devices true -i ""
    // This outputs device names to stderr

    use std::process::Command;

    let output = Command::new("ffmpeg")
        .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
        .output();

    let stderr = match output {
        Ok(out) => String::from_utf8_lossy(&out.stderr).to_string(),
        Err(_) => return vec![],
    };

    parse_avfoundation_device_list(&stderr)
}

/// Parse FFmpeg avfoundation device list output.
#[cfg(target_os = "macos")]
fn parse_avfoundation_device_list(output: &str) -> Vec<WebcamDevice> {
    let mut devices = Vec::new();
    let mut in_video_section = false;

    for line in output.lines() {
        // Look for AVFoundation video devices section
        // Format: [AVFoundation ...] AVFoundation video devices:
        if line.contains("AVFoundation video devices") {
            in_video_section = true;
            continue;
        }

        // Stop at audio devices section
        if line.contains("AVFoundation audio devices") {
            break;
        }

        if !in_video_section {
            continue;
        }

        // Parse device lines
        // Format: [AVFoundation ...] [0] FaceTime HD Camera
        if let Some(bracket_start) = line.find('[') {
            if let Some(bracket_end) = line.find(']') {
                // Check if this is a device index line (not the header)
                let inside_bracket = &line[bracket_start + 1..bracket_end];
                if let Ok(index) = inside_bracket.parse::<usize>() {
                    // Get the device name after the bracket
                    let name = line[bracket_end + 1..].trim().to_string();
                    if !name.is_empty() {
                        devices.push(WebcamDevice {
                            id: index.to_string(),
                            name: name.clone(),
                            description: format!("AVFoundation device {index}"),
                        });
                    }
                }
            }
        }
    }

    devices
}

/// Windows: Query DirectShow devices using FFmpeg.
#[cfg(target_os = "windows")]
fn list_webcams_windows() -> Vec<WebcamDevice> {
    // Use FFmpeg to enumerate DirectShow devices
    // Run: ffmpeg -list_devices true -f dshow -i dummy
    // This outputs device names to stderr
    //
    // We need to hide the console window to prevent a flash when running from GUI apps

    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = Command::new("ffmpeg")
        .args(["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) => {
            // FFmpeg outputs device list to stderr
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            tracing::debug!("FFmpeg dshow output ({} bytes): {}", stderr.len(),
                if stderr.len() > 200 { &stderr[..200] } else { &stderr });
            parse_dshow_device_list(&stderr)
        }
        Err(e) => {
            tracing::debug!("Failed to run ffmpeg for device enumeration: {}", e);
            vec![]
        }
    }
}

/// Parse FFmpeg dshow device list output.
#[cfg(target_os = "windows")]
fn parse_dshow_device_list(output: &str) -> Vec<WebcamDevice> {
    let mut devices = Vec::new();

    for line in output.lines() {
        // Skip alternative name lines
        if line.contains("Alternative name") {
            continue;
        }

        // Look for video devices: [dshow @ ...] "Device Name" (video)
        // Also accept (none) as some virtual cameras report this
        if !line.contains("(video)") && !line.contains("(none)") {
            continue;
        }

        // Extract device name from quotes
        // Format: [dshow @ 0000028e9c143cc0] "USB 2.0 Camera" (video)
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                if end > start {
                    let name = line[start + 1..end].to_string();

                    devices.push(WebcamDevice {
                        id: format!("video={}", name),
                        name: name.clone(),
                        description: "DirectShow video device".to_string(),
                    });
                }
            }
        }
    }

    devices
}

// ============================================================================
// SendableScaler (Thread Safety Wrapper)
// ============================================================================

// Wrapper to make ScalingContext Send-safe
struct SendableScaler(ScalingContext);

// SAFETY: ScalingContext contains raw pointers to FFmpeg structures.
// We ensure thread safety because:
// 1. WebcamPlayer owns SendableScaler exclusively (no sharing)
// 2. WebcamPlayer is Send but not Sync (can move, can't share)
// 3. The scaler is only used within WebcamPlayer's methods
// 4. Only one thread can access the scaler at any time
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for SendableScaler {}

// ============================================================================
// WebcamPlayer (AC: #1, #6)
// ============================================================================

/// Live webcam capture player implementing the [`MediaPlayer`] trait.
///
/// `WebcamPlayer` provides frame-by-frame access to live webcam feeds using
/// FFmpeg for device capture. It supports all major platforms and webcam types.
///
/// # Live Stream Behavior
///
/// Unlike file-based media players, `WebcamPlayer` is a live stream:
/// - `next_frame()` blocks until a new frame is available
/// - `reset()` is a no-op (can't reset live streams)
/// - `frame_count()` returns `None` (unbounded stream)
/// - `loop_count()` returns `Some(0)` (infinite)
///
/// # Examples
///
/// ```no_run
/// use dotmax::media::{WebcamPlayer, MediaPlayer};
///
/// let mut player = WebcamPlayer::new()?;
/// println!("Capturing at {} fps", player.fps());
///
/// // Capture 100 frames
/// for _ in 0..100 {
///     if let Some(Ok((grid, _))) = player.next_frame() {
///         // Process grid...
///     }
/// }
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
pub struct WebcamPlayer {
    /// Device identifier (for error messages).
    device_id: String,

    /// FFmpeg input context (device capture).
    input_context: ffmpeg::format::context::Input,

    /// Video stream index.
    video_stream_index: usize,

    /// Video decoder.
    decoder: ffmpeg::decoder::Video,

    /// Scaler for RGB conversion.
    scaler: SendableScaler,

    /// Capture width in pixels.
    width: u32,

    /// Capture height in pixels.
    height: u32,

    /// Frame rate (fps).
    fps: f64,

    /// Terminal dimensions for rendering.
    terminal_width: usize,
    terminal_height: usize,

    /// Reusable frame buffers.
    decoded_frame: VideoFrame,
    rgb_frame: VideoFrame,

    /// Reusable RGB data buffer.
    rgb_buffer: Vec<u8>,

    // ========== Render Settings ==========
    /// Dithering algorithm.
    dithering: DitheringMethod,

    /// Manual threshold (0-255) or None for Otsu.
    threshold: Option<u8>,

    /// Brightness adjustment.
    brightness: f32,

    /// Contrast adjustment.
    contrast: f32,

    /// Gamma correction.
    gamma: f32,

    /// Color mode.
    color_mode: ColorMode,
}

impl std::fmt::Debug for WebcamPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebcamPlayer")
            .field("device_id", &self.device_id)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("fps", &self.fps)
            .field("dithering", &self.dithering)
            .field("threshold", &self.threshold)
            .field("brightness", &self.brightness)
            .field("contrast", &self.contrast)
            .field("gamma", &self.gamma)
            .field("color_mode", &self.color_mode)
            .finish_non_exhaustive()
    }
}

impl WebcamPlayer {
    /// Creates a new `WebcamPlayer` using the default system webcam.
    ///
    /// # Errors
    ///
    /// Returns `DotmaxError::WebcamError` or specific camera errors if:
    /// - No webcam is detected (`CameraNotFound`)
    /// - Camera is in use (`CameraInUse`)
    /// - Permission denied (`CameraPermissionDenied`)
    /// - FFmpeg initialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dotmax::media::WebcamPlayer;
    ///
    /// let player = WebcamPlayer::new()?;
    /// println!("Webcam opened: {}x{} @ {} fps",
    ///     player.width(), player.height(), player.fps());
    /// # Ok::<(), dotmax::DotmaxError>(())
    /// ```
    pub fn new() -> Result<Self> {
        Self::from_device(WebcamDeviceId::Default)
    }

    /// Creates a new `WebcamPlayer` from a specific device.
    ///
    /// # Arguments
    ///
    /// * `device` - Device identifier (index, path, or name)
    ///
    /// # Errors
    ///
    /// Returns camera-specific errors for invalid device, permission issues,
    /// or device in use.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dotmax::media::WebcamPlayer;
    ///
    /// // By index
    /// let player = WebcamPlayer::from_device(0)?;
    ///
    /// // By path (Linux)
    /// let player = WebcamPlayer::from_device("/dev/video1")?;
    ///
    /// // By name (Windows/macOS)
    /// let player = WebcamPlayer::from_device("FaceTime HD Camera")?;
    /// # Ok::<(), dotmax::DotmaxError>(())
    /// ```
    pub fn from_device(device: impl Into<WebcamDeviceId>) -> Result<Self> {
        let device_id = device.into();
        Self::open_device(device_id, None, None, None)
    }

    /// Returns a builder for configuring the webcam player.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use dotmax::media::WebcamPlayer;
    /// use dotmax::image::DitheringMethod;
    ///
    /// let player = WebcamPlayer::builder()
    ///     .device(0)
    ///     .resolution(1280, 720)
    ///     .fps(30)
    ///     .dithering(DitheringMethod::FloydSteinberg)
    ///     .build()?;
    /// # Ok::<(), dotmax::DotmaxError>(())
    /// ```
    #[must_use]
    pub fn builder() -> WebcamPlayerBuilder {
        WebcamPlayerBuilder::new()
    }

    /// Opens the webcam device with FFmpeg.
    fn open_device(
        device_id: WebcamDeviceId,
        requested_resolution: Option<(u32, u32)>,
        requested_fps: Option<u32>,
        render_settings: Option<RenderSettings>,
    ) -> Result<Self> {
        let device_str = device_id.to_string();

        // Build device URL BEFORE initializing FFmpeg library
        // This is important on Windows where we spawn ffmpeg subprocess to enumerate devices
        // and the ffmpeg library initialization can interfere with that
        let (device_url, input_format) = build_device_url(&device_id)?;
        tracing::debug!("Opening device URL: {} with format: {}", device_url, input_format);

        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| DotmaxError::WebcamError {
            device: device_str.clone(),
            message: format!("FFmpeg initialization failed: {e}"),
        })?;

        // Create input options
        let mut options = ffmpeg::Dictionary::new();

        // Set requested resolution if specified
        if let Some((width, height)) = requested_resolution {
            options.set("video_size", &format!("{width}x{height}"));
        }

        // Set requested frame rate if specified
        if let Some(fps) = requested_fps {
            options.set("framerate", &fps.to_string());
        }

        // Platform-specific options
        #[cfg(target_os = "linux")]
        {
            options.set("input_format", "mjpeg"); // Prefer MJPEG for better performance
        }

        #[cfg(target_os = "windows")]
        {
            // Increase real-time buffer size to prevent overflow warnings
            // Default is ~3MB, increase to 100MB for smoother capture
            options.set("rtbufsize", "100M");
            // Use lower latency settings
            options.set("fflags", "nobuffer");
            options.set("flags", "low_delay");
        }

        // Find the input format by iterating over video devices
        let format = ffmpeg::device::input::video()
            .find(|f| f.name() == input_format)
            .ok_or_else(|| {
                DotmaxError::WebcamError {
                    device: device_str.clone(),
                    message: format!("Input format '{}' not found - FFmpeg may not support webcam capture on this platform", input_format),
                }
            })?;

        // Open device with the correct input format and options
        let context = ffmpeg::format::open_with(&device_url, &format, options)
            .map_err(|e| map_ffmpeg_error(&device_str, e))?;

        // Extract input context from the generic context
        let input_context = match context {
            ffmpeg::format::context::Context::Input(input) => input,
            _ => {
                return Err(DotmaxError::WebcamError {
                    device: device_str,
                    message: "Unexpected output context when opening webcam".to_string(),
                });
            }
        };

        // Find video stream
        let video_stream = input_context
            .streams()
            .best(Type::Video)
            .ok_or_else(|| DotmaxError::WebcamError {
                device: device_str.clone(),
                message: "No video stream found from webcam".to_string(),
            })?;

        let video_stream_index = video_stream.index();

        // Create decoder
        let codec_params = video_stream.parameters();
        let context = ffmpeg::codec::context::Context::from_parameters(codec_params)
            .map_err(|e| DotmaxError::WebcamError {
                device: device_str.clone(),
                message: format!("Failed to create codec context: {e}"),
            })?;

        let decoder = context.decoder().video().map_err(|e| DotmaxError::WebcamError {
            device: device_str.clone(),
            message: format!("Failed to create video decoder: {e}"),
        })?;

        let width = decoder.width();
        let height = decoder.height();

        // Get frame rate
        let fps = video_stream.avg_frame_rate();
        let fps = if fps.denominator() != 0 {
            f64::from(fps.numerator()) / f64::from(fps.denominator())
        } else {
            30.0 // Default to 30 fps
        };

        // Get terminal size
        let (terminal_width, terminal_height) = crossterm::terminal::size()
            .map(|(w, h)| (w as usize, h as usize))
            .unwrap_or((80, 24));

        // Calculate target pixel dimensions for braille grid
        let target_pixel_width = (terminal_width * 2) as u32;
        let target_pixel_height = (terminal_height * 4) as u32;

        // Create scaler
        let scaler = SendableScaler(
            ScalingContext::get(
                decoder.format(),
                width,
                height,
                Pixel::RGB24,
                target_pixel_width,
                target_pixel_height,
                Flags::BILINEAR,
            )
            .map_err(|e| DotmaxError::WebcamError {
                device: device_str.clone(),
                message: format!("Failed to create scaler: {e}"),
            })?,
        );

        tracing::info!(
            "Opened webcam: {}, {}x{} @ {:.2} fps",
            device_str,
            width,
            height,
            fps
        );

        // Pre-allocate RGB buffer
        let rgb_buffer_size = (target_pixel_width * target_pixel_height * 3) as usize;

        // Apply render settings or use defaults
        let settings = render_settings.unwrap_or_default();

        Ok(Self {
            device_id: device_str,
            input_context,
            video_stream_index,
            decoder,
            scaler,
            width,
            height,
            fps,
            terminal_width,
            terminal_height,
            decoded_frame: VideoFrame::empty(),
            rgb_frame: VideoFrame::empty(),
            rgb_buffer: vec![0u8; rgb_buffer_size],
            dithering: settings.dithering,
            threshold: settings.threshold,
            brightness: settings.brightness,
            contrast: settings.contrast,
            gamma: settings.gamma,
            color_mode: settings.color_mode,
        })
    }

    /// Returns the capture width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the capture height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the frame rate (frames per second).
    #[must_use]
    pub const fn fps(&self) -> f64 {
        self.fps
    }

    /// Returns the device identifier.
    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    // ========== Render Settings Builder Methods ==========

    /// Sets the dithering algorithm.
    #[must_use]
    pub const fn dithering(mut self, method: DitheringMethod) -> Self {
        self.dithering = method;
        self
    }

    /// Sets the manual threshold (0-255) or None for automatic Otsu.
    #[must_use]
    pub const fn threshold(mut self, threshold: Option<u8>) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets the brightness adjustment factor.
    #[must_use]
    pub const fn brightness(mut self, brightness: f32) -> Self {
        self.brightness = brightness;
        self
    }

    /// Sets the contrast adjustment factor.
    #[must_use]
    pub const fn contrast(mut self, contrast: f32) -> Self {
        self.contrast = contrast;
        self
    }

    /// Sets the gamma correction factor.
    #[must_use]
    pub const fn gamma(mut self, gamma: f32) -> Self {
        self.gamma = gamma;
        self
    }

    /// Sets the color mode for rendering.
    #[must_use]
    pub const fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    // ========== Getters ==========

    /// Returns the current dithering method.
    #[must_use]
    pub const fn get_dithering(&self) -> DitheringMethod {
        self.dithering
    }

    /// Returns the current threshold setting.
    #[must_use]
    pub const fn get_threshold(&self) -> Option<u8> {
        self.threshold
    }

    /// Returns the current brightness setting.
    #[must_use]
    pub const fn get_brightness(&self) -> f32 {
        self.brightness
    }

    /// Returns the current contrast setting.
    #[must_use]
    pub const fn get_contrast(&self) -> f32 {
        self.contrast
    }

    /// Returns the current gamma setting.
    #[must_use]
    pub const fn get_gamma(&self) -> f32 {
        self.gamma
    }

    /// Returns the current color mode.
    #[must_use]
    pub const fn get_color_mode(&self) -> ColorMode {
        self.color_mode
    }

    // ========== Mutable setters ==========

    /// Updates the dithering method at runtime.
    pub fn set_dithering(&mut self, method: DitheringMethod) {
        self.dithering = method;
    }

    /// Updates the threshold at runtime.
    pub fn set_threshold(&mut self, threshold: Option<u8>) {
        self.threshold = threshold;
    }

    /// Updates the brightness at runtime.
    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness;
    }

    /// Updates the contrast at runtime.
    pub fn set_contrast(&mut self, contrast: f32) {
        self.contrast = contrast;
    }

    /// Updates the gamma at runtime.
    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma;
    }

    /// Updates the color mode at runtime.
    pub fn set_color_mode(&mut self, mode: ColorMode) {
        self.color_mode = mode;
    }

    /// Decodes the next frame from the webcam.
    fn decode_next_frame(&mut self) -> Option<Result<()>> {
        // Try to receive a decoded frame
        loop {
            // First, try to receive from decoder
            match self.decoder.receive_frame(&mut self.decoded_frame) {
                Ok(()) => {
                    return Some(Ok(()));
                }
                Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::error::EAGAIN => {
                    // Decoder needs more data
                }
                Err(e) => {
                    return Some(Err(DotmaxError::WebcamError {
                        device: self.device_id.clone(),
                        message: format!("Frame decode error: {e}"),
                    }));
                }
            }

            // Read next packet from device
            let mut found_video_packet = false;
            for (stream, packet) in self.input_context.packets() {
                if stream.index() == self.video_stream_index {
                    if let Err(e) = self.decoder.send_packet(&packet) {
                        tracing::warn!("Error sending packet to decoder: {}", e);
                    }
                    found_video_packet = true;
                    break;
                }
            }

            if !found_video_packet {
                // No packet available - for live streams this shouldn't happen
                // but we'll handle it gracefully
                return Some(Err(DotmaxError::WebcamError {
                    device: self.device_id.clone(),
                    message: "Webcam stream ended unexpectedly".to_string(),
                }));
            }
        }
    }

    /// Converts the decoded frame to a BrailleGrid.
    ///
    /// Optimized pipeline:
    /// 1. FFmpeg scaler already resizes to terminal pixel dimensions
    /// 2. Direct RGB→grayscale conversion (no RGBA intermediate)
    /// 3. Reuse pre-allocated buffers
    /// 4. Skip redundant resize in ImageRenderer
    fn frame_to_grid(&mut self) -> Result<BrailleGrid> {
        // Scale to RGB24 at terminal dimensions (FFmpeg hardware-accelerated)
        self.scaler
            .0
            .run(&self.decoded_frame, &mut self.rgb_frame)
            .map_err(|e| DotmaxError::WebcamError {
                device: self.device_id.clone(),
                message: format!("Frame scaling error: {e}"),
            })?;

        // Get RGB data directly from FFmpeg frame
        let data = self.rgb_frame.data(0);
        let stride = self.rgb_frame.stride(0);
        let target_width = (self.terminal_width * 2) as u32;
        let target_height = (self.terminal_height * 4) as u32;

        // Fast path: if stride matches width*3, data is contiguous
        let row_bytes = (target_width as usize) * 3;
        let rgb_data: &[u8] = if stride as usize == row_bytes {
            // Contiguous data - use directly without copy
            &data[..row_bytes * (target_height as usize)]
        } else {
            // Non-contiguous - copy row by row into buffer
            let expected_size = (target_width * target_height * 3) as usize;
            if self.rgb_buffer.len() != expected_size {
                self.rgb_buffer.resize(expected_size, 0);
            }

            let mut offset = 0;
            for y in 0..target_height {
                let row_start = (y as usize) * (stride as usize);
                self.rgb_buffer[offset..offset + row_bytes]
                    .copy_from_slice(&data[row_start..row_start + row_bytes]);
                offset += row_bytes;
            }
            &self.rgb_buffer
        };

        // Create RGB image - we need ownership for the image crate
        let img = image::RgbImage::from_raw(target_width, target_height, rgb_data.to_vec())
            .ok_or_else(|| DotmaxError::WebcamError {
                device: self.device_id.clone(),
                message: "Failed to create image from frame data".to_string(),
            })?;

        let dynamic_img = image::DynamicImage::ImageRgb8(img);

        // Convert to grayscale for braille pattern generation
        let gray = crate::image::to_grayscale(&dynamic_img);

        // Apply brightness/contrast/gamma adjustments if needed
        let adjusted_gray = self.apply_adjustments(gray)?;

        // Apply dithering and convert to braille
        self.gray_to_braille_grid_with_color(adjusted_gray, &dynamic_img, target_width, target_height)
    }

    /// Applies brightness, contrast, and gamma adjustments to grayscale image.
    #[inline]
    fn apply_adjustments(&self, mut gray: image::GrayImage) -> Result<image::GrayImage> {
        if (self.brightness - 1.0).abs() > f32::EPSILON {
            gray = crate::image::adjust_brightness(&gray, self.brightness)?;
        }
        if (self.contrast - 1.0).abs() > f32::EPSILON {
            gray = crate::image::adjust_contrast(&gray, self.contrast)?;
        }
        if (self.gamma - 1.0).abs() > f32::EPSILON {
            gray = crate::image::adjust_gamma(&gray, self.gamma)?;
        }
        Ok(gray)
    }

    /// Converts grayscale image to BrailleGrid using configured dithering, with color support.
    fn gray_to_braille_grid_with_color(
        &self,
        gray: image::GrayImage,
        rgb_image: &image::DynamicImage,
        width: u32,
        height: u32,
    ) -> Result<BrailleGrid> {
        use crate::image::{
            apply_dithering_with_custom_threshold, apply_threshold, otsu_threshold,
            pixels_to_braille, DitheringMethod,
        };
        use crate::image::color_mode::{extract_cell_colors, ColorSamplingStrategy};

        // Get binary image via dithering or thresholding
        let binary = match self.dithering {
            DitheringMethod::None => {
                // Simple thresholding
                let thresh = self.threshold.unwrap_or_else(|| otsu_threshold(&gray));
                apply_threshold(&gray, thresh)
            }
            _ => {
                // Apply dithering algorithm
                apply_dithering_with_custom_threshold(&gray, self.dithering, self.threshold)?
            }
        };

        // Convert to braille grid (dimensions in braille cells = pixels / 2x4)
        let grid_width = (width as usize) / 2;
        let grid_height = (height as usize) / 4;
        let mut grid = pixels_to_braille(&binary, grid_width, grid_height)?;

        // Apply colors if not monochrome mode
        match self.color_mode {
            ColorMode::Monochrome => {}
            ColorMode::TrueColor | ColorMode::Grayscale => {
                // Extract colors from original RGB image
                let colors = extract_cell_colors(
                    rgb_image,
                    grid_width,
                    grid_height,
                    ColorSamplingStrategy::Average,
                );

                // Enable color support and apply colors to grid
                grid.enable_color_support();
                for (idx, color) in colors.into_iter().enumerate() {
                    let x = idx % grid_width;
                    let y = idx / grid_width;

                    // For grayscale mode, convert color to gray
                    let final_color = if self.color_mode == ColorMode::Grayscale {
                        let gray_val = ((color.r as u32 + color.g as u32 + color.b as u32) / 3) as u8;
                        crate::Color { r: gray_val, g: gray_val, b: gray_val }
                    } else {
                        color
                    };

                    let _ = grid.set_cell_color(x, y, final_color);
                }
            }
        }

        Ok(grid)
    }

    /// Calculates the delay for frame timing.
    fn frame_delay(&self) -> Duration {
        if self.fps > 0.0 {
            Duration::from_secs_f64(1.0 / self.fps)
        } else {
            Duration::from_millis(33) // ~30 fps default
        }
    }
}

// ============================================================================
// MediaPlayer Implementation (AC: #1)
// ============================================================================

impl MediaPlayer for WebcamPlayer {
    /// Returns the next frame and its display duration.
    ///
    /// For webcams, this blocks until a new frame is captured.
    fn next_frame(&mut self) -> Option<Result<(BrailleGrid, Duration)>> {
        // Decode next frame
        match self.decode_next_frame() {
            Some(Ok(())) => {}
            Some(Err(e)) => return Some(Err(e)),
            None => return None,
        }

        // Convert to grid
        let grid = match self.frame_to_grid() {
            Ok(g) => g,
            Err(e) => return Some(Err(e)),
        };

        let delay = self.frame_delay();
        Some(Ok((grid, delay)))
    }

    /// No-op for live webcam streams.
    ///
    /// Webcams are live streams and cannot be reset to the beginning.
    fn reset(&mut self) {
        // No-op for live streams
        tracing::debug!("WebcamPlayer::reset() called - no-op for live streams");
    }

    /// Returns `None` as webcam streams are unbounded.
    fn frame_count(&self) -> Option<usize> {
        None // Live stream - infinite frames
    }

    /// Returns `Some(0)` indicating infinite looping.
    fn loop_count(&self) -> Option<u16> {
        Some(0) // Infinite
    }

    /// Updates terminal dimensions for rendering.
    fn handle_resize(&mut self, width: usize, height: usize) {
        if self.terminal_width == width && self.terminal_height == height {
            return;
        }

        self.terminal_width = width;
        self.terminal_height = height;

        // Calculate new target dimensions
        let target_pixel_width = (width * 2) as u32;
        let target_pixel_height = (height * 4) as u32;

        // Recreate scaler
        match ScalingContext::get(
            self.decoder.format(),
            self.width,
            self.height,
            Pixel::RGB24,
            target_pixel_width,
            target_pixel_height,
            Flags::BILINEAR,
        ) {
            Ok(new_scaler) => {
                self.scaler = SendableScaler(new_scaler);
                let rgb_buffer_size = (target_pixel_width * target_pixel_height * 3) as usize;
                self.rgb_buffer.resize(rgb_buffer_size, 0);
                tracing::debug!("WebcamPlayer resized to {}x{}", width, height);
            }
            Err(e) => {
                tracing::warn!("Failed to resize webcam scaler: {}", e);
            }
        }
    }
}

// ============================================================================
// WebcamPlayerBuilder (AC: #8)
// ============================================================================

/// Render settings for webcam capture.
#[derive(Debug, Clone)]
struct RenderSettings {
    dithering: DitheringMethod,
    threshold: Option<u8>,
    brightness: f32,
    contrast: f32,
    gamma: f32,
    color_mode: ColorMode,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            dithering: DitheringMethod::FloydSteinberg,
            threshold: None,
            brightness: 1.0,
            contrast: 1.0,
            gamma: 1.0,
            color_mode: ColorMode::Monochrome,
        }
    }
}

/// Builder for configuring [`WebcamPlayer`].
///
/// Use this builder to customize webcam capture settings before opening
/// the device.
///
/// # Examples
///
/// ```no_run
/// use dotmax::media::WebcamPlayer;
/// use dotmax::image::{DitheringMethod, ColorMode};
///
/// let player = WebcamPlayer::builder()
///     .device(0)  // First camera
///     .resolution(1280, 720)
///     .fps(30)
///     .dithering(DitheringMethod::Bayer)
///     .color_mode(ColorMode::TrueColor)
///     .build()?;
/// # Ok::<(), dotmax::DotmaxError>(())
/// ```
#[derive(Debug, Default)]
pub struct WebcamPlayerBuilder {
    device: WebcamDeviceId,
    resolution: Option<(u32, u32)>,
    fps: Option<u32>,
    render_settings: RenderSettings,
}

impl WebcamPlayerBuilder {
    /// Creates a new builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the device to capture from.
    ///
    /// Accepts device index, path, or name.
    #[must_use]
    pub fn device(mut self, device: impl Into<WebcamDeviceId>) -> Self {
        self.device = device.into();
        self
    }

    /// Sets the requested capture resolution.
    ///
    /// Note: The actual resolution may differ if the camera doesn't support
    /// the requested size.
    #[must_use]
    pub const fn resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = Some((width, height));
        self
    }

    /// Sets the requested frame rate.
    ///
    /// Note: The actual frame rate may differ if the camera doesn't support
    /// the requested rate.
    #[must_use]
    pub const fn fps(mut self, fps: u32) -> Self {
        self.fps = Some(fps);
        self
    }

    /// Sets the dithering algorithm.
    #[must_use]
    pub const fn dithering(mut self, method: DitheringMethod) -> Self {
        self.render_settings.dithering = method;
        self
    }

    /// Sets the manual threshold (0-255) or None for Otsu.
    #[must_use]
    pub const fn threshold(mut self, threshold: Option<u8>) -> Self {
        self.render_settings.threshold = threshold;
        self
    }

    /// Sets the brightness adjustment.
    #[must_use]
    pub const fn brightness(mut self, brightness: f32) -> Self {
        self.render_settings.brightness = brightness;
        self
    }

    /// Sets the contrast adjustment.
    #[must_use]
    pub const fn contrast(mut self, contrast: f32) -> Self {
        self.render_settings.contrast = contrast;
        self
    }

    /// Sets the gamma correction.
    #[must_use]
    pub const fn gamma(mut self, gamma: f32) -> Self {
        self.render_settings.gamma = gamma;
        self
    }

    /// Sets the color mode.
    #[must_use]
    pub const fn color_mode(mut self, mode: ColorMode) -> Self {
        self.render_settings.color_mode = mode;
        self
    }

    /// Builds the `WebcamPlayer` with the configured settings.
    ///
    /// # Errors
    ///
    /// Returns camera-specific errors if the device cannot be opened.
    pub fn build(self) -> Result<WebcamPlayer> {
        WebcamPlayer::open_device(
            self.device,
            self.resolution,
            self.fps,
            Some(self.render_settings),
        )
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Builds the device URL and input format for the current platform.
#[allow(clippy::unnecessary_wraps)] // Returns Result for unsupported platform case
fn build_device_url(device_id: &WebcamDeviceId) -> Result<(String, &'static str)> {
    #[cfg(target_os = "linux")]
    {
        let device_path = match device_id {
            WebcamDeviceId::Default => "/dev/video0".to_string(),
            WebcamDeviceId::Index(i) => format!("/dev/video{i}"),
            WebcamDeviceId::Path(p) => p.clone(),
        };
        Ok((device_path, "v4l2"))
    }

    #[cfg(target_os = "macos")]
    {
        let device_index = match device_id {
            WebcamDeviceId::Default => "0".to_string(),
            WebcamDeviceId::Index(i) => i.to_string(),
            WebcamDeviceId::Path(p) => p.clone(),
        };
        Ok((device_index, "avfoundation"))
    }

    #[cfg(target_os = "windows")]
    {
        let device_name = match device_id {
            WebcamDeviceId::Default => {
                // Get the first available camera
                let devices = list_webcams();
                if devices.is_empty() {
                    return Err(DotmaxError::CameraNotFound {
                        device: "default".to_string(),
                        available: vec![],
                    });
                }
                devices[0].id.clone()
            }
            WebcamDeviceId::Index(i) => {
                // Look up device by index from enumerated list
                let devices = list_webcams();
                tracing::debug!("Windows device lookup: index={}, found {} devices", i, devices.len());
                if *i >= devices.len() {
                    // Re-enumerate for error message (the first call may have failed transiently)
                    let devices_retry = list_webcams();
                    let available: Vec<String> = devices_retry.iter().map(|d| d.name.clone()).collect();
                    return Err(DotmaxError::CameraNotFound {
                        device: format!("index:{i}"),
                        available,
                    });
                }
                devices[*i].id.clone()
            }
            WebcamDeviceId::Path(p) => {
                if p.starts_with("video=") {
                    p.clone()
                } else {
                    format!("video={p}")
                }
            }
        };
        Ok((device_name, "dshow"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(DotmaxError::WebcamError {
            device: device_id.to_string(),
            message: "Webcam capture not supported on this platform".to_string(),
        })
    }
}

/// Maps FFmpeg errors to appropriate DotmaxError variants.
fn map_ffmpeg_error(device: &str, error: ffmpeg::Error) -> DotmaxError {
    let error_str = error.to_string().to_lowercase();

    // Check for common error patterns
    if error_str.contains("no such file") || error_str.contains("not found") {
        let available: Vec<String> = list_webcams().iter().map(|d| d.name.clone()).collect();
        return DotmaxError::CameraNotFound {
            device: device.to_string(),
            available,
        };
    }

    if error_str.contains("permission") || error_str.contains("access denied") {
        let hint = get_permission_hint();
        return DotmaxError::CameraPermissionDenied {
            device: device.to_string(),
            hint,
        };
    }

    if error_str.contains("busy") || error_str.contains("in use") || error_str.contains("device or resource busy") {
        return DotmaxError::CameraInUse {
            device: device.to_string(),
        };
    }

    // Generic webcam error
    DotmaxError::WebcamError {
        device: device.to_string(),
        message: format!("Failed to open webcam: {error}"),
    }
}

/// Returns platform-specific permission hint.
fn get_permission_hint() -> String {
    #[cfg(target_os = "linux")]
    {
        "Add your user to the 'video' group: sudo usermod -aG video $USER (then log out and back in)".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "Grant camera access in System Preferences > Security & Privacy > Privacy > Camera".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "Grant camera access in Settings > Privacy > Camera".to_string()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "Check your system's camera permission settings".to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure WebcamPlayer is Send (required by MediaPlayer trait)
    fn _assert_webcam_player_send() {
        fn assert_send<T: Send>() {}
        assert_send::<WebcamPlayer>();
    }

    #[test]
    fn test_webcam_device_new() {
        let device = WebcamDevice::new("/dev/video0", "USB Camera", "Generic USB webcam");
        assert_eq!(device.id, "/dev/video0");
        assert_eq!(device.name, "USB Camera");
        assert_eq!(device.description, "Generic USB webcam");
    }

    #[test]
    fn test_webcam_device_id_from_index() {
        let id: WebcamDeviceId = 0.into();
        assert!(matches!(id, WebcamDeviceId::Index(0)));
    }

    #[test]
    fn test_webcam_device_id_from_str() {
        let id: WebcamDeviceId = "/dev/video0".into();
        assert!(matches!(id, WebcamDeviceId::Path(_)));
    }

    #[test]
    fn test_webcam_device_id_from_string() {
        let id: WebcamDeviceId = String::from("/dev/video1").into();
        assert!(matches!(id, WebcamDeviceId::Path(_)));
    }

    #[test]
    fn test_webcam_device_id_display() {
        assert_eq!(WebcamDeviceId::Default.to_string(), "default");
        assert_eq!(WebcamDeviceId::Index(0).to_string(), "index:0");
        assert_eq!(WebcamDeviceId::Path("/dev/video0".into()).to_string(), "/dev/video0");
    }

    #[test]
    fn test_list_webcams_returns_vec() {
        // This will return actual devices or empty vec in CI
        let devices = list_webcams();
        // Just verify it doesn't panic and returns a Vec
        let _ = devices.len();
    }

    #[test]
    fn test_webcam_player_builder_chain() {
        // Test that builder methods compile and chain correctly
        let builder = WebcamPlayerBuilder::new()
            .device(0)
            .resolution(1280, 720)
            .fps(30)
            .dithering(DitheringMethod::FloydSteinberg)
            .threshold(Some(128))
            .brightness(1.2)
            .contrast(1.1)
            .gamma(0.9)
            .color_mode(ColorMode::Monochrome);

        // Verify settings were stored
        assert!(matches!(builder.device, WebcamDeviceId::Index(0)));
        assert_eq!(builder.resolution, Some((1280, 720)));
        assert_eq!(builder.fps, Some(30));
        assert_eq!(builder.render_settings.dithering, DitheringMethod::FloydSteinberg);
        assert_eq!(builder.render_settings.threshold, Some(128));
        assert!((builder.render_settings.brightness - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_render_settings_default() {
        let settings = RenderSettings::default();
        assert_eq!(settings.dithering, DitheringMethod::FloydSteinberg);
        assert_eq!(settings.threshold, None);
        assert!((settings.brightness - 1.0).abs() < f32::EPSILON);
        assert!((settings.contrast - 1.0).abs() < f32::EPSILON);
        assert!((settings.gamma - 1.0).abs() < f32::EPSILON);
        assert_eq!(settings.color_mode, ColorMode::Monochrome);
    }

    // Note: Tests requiring actual webcam hardware are marked #[ignore]
    // and should be run manually with `cargo test -- --ignored`

    #[test]
    #[ignore = "Requires webcam hardware"]
    fn test_webcam_player_new_with_camera() {
        let result = WebcamPlayer::new();
        if let Ok(player) = result {
            assert!(player.width() > 0);
            assert!(player.height() > 0);
            assert!(player.fps() > 0.0);
        }
        // It's okay if this fails - may not have a camera
    }
}
