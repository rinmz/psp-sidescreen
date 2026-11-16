//! Screen capture with Desktop Duplication.
//!
//! Desktop Duplication reports physical pixels and leaves the mouse pointer out of the image,
//! giving its position and shape separately. The pointer is drawn into the frame here, with the
//! output's own coordinates, so it lands where the person sees it whatever the Windows display
//! scaling is.
//!
//! This module calls the Windows API and is the only one in the crate that needs `unsafe`.
#![allow(unsafe_code)]

use std::fmt;

use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11CreateDevice, ID3D11DeviceContext,
    ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT, DXGI_OUTDUPL_FRAME_INFO,
    DXGI_OUTDUPL_POINTER_SHAPE_INFO, DXGI_OUTDUPL_POINTER_SHAPE_TYPE_COLOR, IDXGIAdapter,
    IDXGIAdapter1, IDXGIFactory1, IDXGIOutput, IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource,
};
use windows::core::Interface;

/// Why capture failed.
#[derive(Debug)]
pub enum CaptureError {
    /// No monitor has the requested index.
    NoSuchMonitor(usize),
    /// Windows ended the duplication (a secure desktop, a mode change); open it again.
    AccessLost,
    /// Any other Windows error.
    Windows(windows::core::Error),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchMonitor(index) => write!(f, "there is no monitor {index}"),
            Self::AccessLost => f.write_str("Windows ended the screen capture"),
            Self::Windows(error) => write!(f, "screen capture failed: {error}"),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<windows::core::Error> for CaptureError {
    fn from(error: windows::core::Error) -> Self {
        if error.code() == DXGI_ERROR_ACCESS_LOST {
            Self::AccessLost
        } else {
            Self::Windows(error)
        }
    }
}

/// A monitor that can be captured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitor {
    /// Position in the list, counting across all adapters.
    pub index: usize,
    /// Windows' name for it, such as `\\.\DISPLAY1`.
    pub name: String,
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
    /// Left edge on the virtual desktop.
    pub left: i32,
    /// Top edge on the virtual desktop.
    pub top: i32,
}

/// One captured picture, 32 bits per pixel in B, G, R, A order.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` bytes.
    pub bgra: Vec<u8>,
}

/// Lists the monitors attached to the desktop, in a stable order.
///
/// # Errors
///
/// [`CaptureError`] if DXGI cannot be queried.
pub fn list_monitors() -> Result<Vec<Monitor>, CaptureError> {
    let mut monitors = Vec::new();
    for_each_output(|adapter, output| {
        let desc = unsafe { output.GetDesc()? };
        let rect = desc.DesktopCoordinates;
        let length = desc
            .DeviceName
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(desc.DeviceName.len());
        let _ = adapter;
        monitors.push(Monitor {
            index: monitors.len(),
            name: String::from_utf16_lossy(&desc.DeviceName[..length]),
            width: u32::try_from(rect.right - rect.left).unwrap_or(0),
            height: u32::try_from(rect.bottom - rect.top).unwrap_or(0),
            left: rect.left,
            top: rect.top,
        });
        Ok(true)
    })?;
    Ok(monitors)
}

/// Calls `visit` for every output of every adapter until it returns `false`.
fn for_each_output(
    mut visit: impl FnMut(&IDXGIAdapter1, &IDXGIOutput) -> Result<bool, CaptureError>,
) -> Result<(), CaptureError> {
    // SAFETY: plain COM calls on interfaces this function owns; no pointers escape.
    unsafe {
        let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
        let mut adapter_index = 0;
        while let Ok(adapter) = factory.EnumAdapters1(adapter_index) {
            let mut output_index = 0;
            while let Ok(output) = adapter.EnumOutputs(output_index) {
                if !visit(&adapter, &output)? {
                    return Ok(());
                }
                output_index += 1;
            }
            adapter_index += 1;
        }
    }
    Ok(())
}

/// The shape of the mouse pointer, kept between frames because Windows sends it only when it
/// changes.
#[derive(Default)]
struct Pointer {
    visible: bool,
    x: i32,
    y: i32,
    shape: Option<Shape>,
}

struct Shape {
    width: usize,
    height: usize,
    pitch: usize,
    bgra: Vec<u8>,
}

/// A running capture of one monitor.
pub struct Capturer {
    context: ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
    staging: ID3D11Texture2D,
    width: u32,
    height: u32,
    pointer: Pointer,
}

impl Capturer {
    /// Starts capturing the monitor with the given index.
    ///
    /// # Errors
    ///
    /// [`CaptureError::NoSuchMonitor`] if the index is out of range, otherwise the Windows error
    /// that stopped the duplication from starting (for example another program already holds
    /// it, or the session is locked).
    pub fn open(index: usize) -> Result<Self, CaptureError> {
        let mut found = None;
        let mut current = 0;
        for_each_output(|adapter, output| {
            if current == index {
                found = Some((adapter.clone(), output.clone()));
                return Ok(false);
            }
            current += 1;
            Ok(true)
        })?;
        let (adapter, output) = found.ok_or(CaptureError::NoSuchMonitor(index))?;

        // SAFETY: every out-parameter points at a local that lives through the call, and the
        // interfaces are reference counted by the `windows` crate.
        unsafe {
            let mut device = None;
            let mut context = None;
            let adapter: IDXGIAdapter = adapter.cast()?;
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                windows::Win32::Graphics::Direct3D11::D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&raw mut device),
                None,
                Some(&raw mut context),
            )?;
            let device = device.ok_or_else(windows::core::Error::empty)?;
            let context = context.ok_or_else(windows::core::Error::empty)?;

            let output1: IDXGIOutput1 = output.cast()?;
            let duplication = output1.DuplicateOutput(&device)?;

            let desc = duplication.GetDesc();
            let (width, height) = (desc.ModeDesc.Width, desc.ModeDesc.Height);

            let staging_desc = D3D11_TEXTURE2D_DESC {
                Width: width,
                Height: height,
                MipLevels: 1,
                ArraySize: 1,
                Format: desc.ModeDesc.Format,
                SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_STAGING,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0.cast_unsigned(),
                ..Default::default()
            };
            let mut staging = None;
            device.CreateTexture2D(&raw const staging_desc, None, Some(&raw mut staging))?;
            let staging = staging.ok_or_else(windows::core::Error::empty)?;

            Ok(Self {
                context,
                duplication,
                staging,
                width,
                height,
                pointer: Pointer::default(),
            })
        }
    }

    /// Size of the captured monitor in physical pixels.
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Waits up to `timeout_ms` for the screen to change and copies it into `frame`.
    ///
    /// Returns `false` if nothing changed in that time; `frame` then keeps its old content.
    ///
    /// # Errors
    ///
    /// [`CaptureError::AccessLost`] when Windows ends the duplication, otherwise the Windows
    /// error.
    pub fn next_frame(&mut self, timeout_ms: u32, frame: &mut Frame) -> Result<bool, CaptureError> {
        let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        // SAFETY: out-parameters are locals; the frame is released before returning on every
        // path that acquired it.
        unsafe {
            match self
                .duplication
                .AcquireNextFrame(timeout_ms, &raw mut info, &raw mut resource)
            {
                Ok(()) => {}
                Err(error) if error.code() == DXGI_ERROR_WAIT_TIMEOUT => return Ok(false),
                Err(error) => return Err(error.into()),
            }
            let result = self.read_frame(&info, resource, frame);
            // Releasing can fail only if the duplication is already lost; the next acquire
            // reports that.
            let _ = self.duplication.ReleaseFrame();
            result?;
        }
        Ok(true)
    }

    /// Copies the acquired frame to the CPU and updates the pointer.
    ///
    /// # Safety
    ///
    /// A frame must be acquired and not yet released.
    unsafe fn read_frame(
        &mut self,
        info: &DXGI_OUTDUPL_FRAME_INFO,
        resource: Option<IDXGIResource>,
        frame: &mut Frame,
    ) -> Result<(), CaptureError> {
        // SAFETY: the caller holds an acquired frame; every pointer below is to a local or to
        // memory that `Map` returned and that stays valid until `Unmap`.
        unsafe {
            if info.LastPresentTime != 0 {
                let texture: ID3D11Texture2D =
                    resource.ok_or_else(windows::core::Error::empty)?.cast()?;
                self.context.CopyResource(&self.staging, &texture);

                let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                self.context
                    .Map(&self.staging, 0, D3D11_MAP_READ, 0, Some(&raw mut mapped))?;
                let row_bytes = self.width as usize * 4;
                frame.width = self.width;
                frame.height = self.height;
                frame.bgra.resize(row_bytes * self.height as usize, 0);
                let source = mapped.pData.cast::<u8>().cast_const();
                for (row, out) in frame.bgra.chunks_exact_mut(row_bytes).enumerate() {
                    let line = std::slice::from_raw_parts(
                        source.add(row * mapped.RowPitch as usize),
                        row_bytes,
                    );
                    out.copy_from_slice(line);
                }
                self.context.Unmap(&self.staging, 0);
            }
            self.update_pointer(info)?;
        }
        draw_pointer(frame, &self.pointer);
        Ok(())
    }

    /// Reads the pointer's position and, when it changed, its shape.
    ///
    /// # Safety
    ///
    /// A frame must be acquired and not yet released.
    unsafe fn update_pointer(
        &mut self,
        info: &DXGI_OUTDUPL_FRAME_INFO,
    ) -> Result<(), CaptureError> {
        if info.LastMouseUpdateTime != 0 {
            self.pointer.visible = info.PointerPosition.Visible.as_bool();
            self.pointer.x = info.PointerPosition.Position.x;
            self.pointer.y = info.PointerPosition.Position.y;
        }
        if info.PointerShapeBufferSize == 0 {
            return Ok(());
        }
        let mut buffer = vec![0_u8; info.PointerShapeBufferSize as usize];
        let mut required = 0;
        let mut shape_info = DXGI_OUTDUPL_POINTER_SHAPE_INFO::default();
        // SAFETY: `buffer` is as large as the size Windows reported for it.
        unsafe {
            self.duplication.GetFramePointerShape(
                info.PointerShapeBufferSize,
                buffer.as_mut_ptr().cast(),
                &raw mut required,
                &raw mut shape_info,
            )?;
        }
        // Only colour pointers are drawn. Monochrome and masked ones need an XOR with the
        // screen underneath; they are rare in current Windows and are left out.
        self.pointer.shape = (shape_info.Type
            == DXGI_OUTDUPL_POINTER_SHAPE_TYPE_COLOR.0.cast_unsigned())
        .then_some(Shape {
            width: shape_info.Width as usize,
            height: shape_info.Height as usize,
            pitch: shape_info.Pitch as usize,
            bgra: buffer,
        });
        Ok(())
    }
}

/// Blends the colour pointer into `frame` at its position, clipped to the frame.
fn draw_pointer(frame: &mut Frame, pointer: &Pointer) {
    let Pointer {
        visible: true,
        x,
        y,
        shape: Some(shape),
    } = pointer
    else {
        return;
    };
    let (frame_width, frame_height) = (i64::from(frame.width), i64::from(frame.height));
    for row in 0..shape.height {
        let target_y = i64::from(*y) + i64::try_from(row).unwrap_or(i64::MAX);
        if !(0..frame_height).contains(&target_y) {
            continue;
        }
        for column in 0..shape.width {
            let target_x = i64::from(*x) + i64::try_from(column).unwrap_or(i64::MAX);
            if !(0..frame_width).contains(&target_x) {
                continue;
            }
            let source = &shape.bgra[row * shape.pitch + column * 4..][..4];
            let alpha = u32::from(source[3]);
            if alpha == 0 {
                continue;
            }
            // Both coordinates are inside the frame, so the index is non-negative.
            let offset = usize::try_from(target_y * frame_width + target_x).unwrap_or(0) * 4;
            for channel in 0..3 {
                let below = u32::from(frame.bgra[offset + channel]);
                let above = u32::from(source[channel]);
                // Windows hands the colour pointer with straight alpha.
                #[allow(clippy::cast_possible_truncation)]
                let blended = ((above * alpha + below * (255 - alpha)) / 255) as u8;
                frame.bgra[offset + channel] = blended;
            }
        }
    }
}

impl fmt::Debug for Capturer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Capturer")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A black frame.
    fn blank(width: u32, height: u32) -> Frame {
        Frame {
            width,
            height,
            bgra: vec![0; (width * height * 4) as usize],
        }
    }

    /// A 2x2 pointer; the top-left pixel is opaque white, the top-right is half-transparent
    /// white, the bottom row is fully transparent.
    fn pointer_at(x: i32, y: i32) -> Pointer {
        let mut bgra = vec![0; 2 * 2 * 4];
        bgra[0..4].copy_from_slice(&[255, 255, 255, 255]);
        bgra[4..8].copy_from_slice(&[255, 255, 255, 128]);
        Pointer {
            visible: true,
            x,
            y,
            shape: Some(Shape {
                width: 2,
                height: 2,
                pitch: 8,
                bgra,
            }),
        }
    }

    fn pixel(frame: &Frame, x: u32, y: u32) -> [u8; 3] {
        let offset = ((y * frame.width + x) * 4) as usize;
        [
            frame.bgra[offset],
            frame.bgra[offset + 1],
            frame.bgra[offset + 2],
        ]
    }

    #[test]
    fn opaque_pixel_replaces_the_screen_and_half_alpha_mixes() {
        let mut frame = blank(4, 4);
        draw_pointer(&mut frame, &pointer_at(1, 1));
        assert_eq!(pixel(&frame, 1, 1), [255, 255, 255]);
        assert_eq!(pixel(&frame, 2, 1), [128, 128, 128]);
    }

    #[test]
    fn transparent_pixels_leave_the_screen_alone() {
        let mut frame = blank(4, 4);
        draw_pointer(&mut frame, &pointer_at(1, 1));
        assert_eq!(pixel(&frame, 1, 2), [0, 0, 0]);
        assert_eq!(pixel(&frame, 0, 0), [0, 0, 0]);
    }

    #[test]
    fn pointer_partly_off_the_frame_is_clipped_without_panicking() {
        let mut frame = blank(4, 4);
        draw_pointer(&mut frame, &pointer_at(-1, 3));
        draw_pointer(&mut frame, &pointer_at(3, 3));
        draw_pointer(&mut frame, &pointer_at(100, 100));
        assert_eq!(pixel(&frame, 3, 3), [255, 255, 255]);
        assert_eq!(pixel(&frame, 0, 3), [128, 128, 128]);
    }

    #[test]
    fn hidden_pointer_is_not_drawn() {
        let mut frame = blank(4, 4);
        let mut pointer = pointer_at(0, 0);
        pointer.visible = false;
        draw_pointer(&mut frame, &pointer);
        assert_eq!(pixel(&frame, 0, 0), [0, 0, 0]);
    }
}
