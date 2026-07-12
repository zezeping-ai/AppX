//! 从系统提取 App 图标并编码为 PNG 字节。

pub fn fetch_icon_bytes(app_id: &str) -> Result<Option<Vec<u8>>, String> {
    #[cfg(target_os = "macos")]
    {
        macos_fetch_icon_bytes(app_id)
    }
    #[cfg(target_os = "windows")]
    {
        windows_fetch_icon_bytes(app_id)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if super::foreground::is_x11_session() {
            linux_fetch_icon_bytes(app_id)
        } else {
            Ok(None)
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_fetch_icon_bytes(bundle: &str) -> Result<Option<Vec<u8>>, String> {
    use std::io::Cursor;

    use image::ImageFormat;
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::NSString;

    let workspace = NSWorkspace::sharedWorkspace();
    let ns_bundle = NSString::from_str(bundle);
    let Some(app_url) = workspace.URLForApplicationWithBundleIdentifier(&ns_bundle) else {
        return Ok(None);
    };
    let Some(app_path) = app_url.path() else {
        return Ok(None);
    };
    let icon = workspace.iconForFile(&app_path);
    let Some(tiff) = icon.TIFFRepresentation() else {
        return Ok(None);
    };
    let bytes: Vec<u8> = tiff.to_vec();
    let img = image::load_from_memory(&bytes).map_err(|e| format!("解码图标失败：{e}"))?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| format!("编码 PNG 失败：{e}"))?;
    Ok(Some(buf))
}

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::HICON;

#[cfg(target_os = "windows")]
fn windows_fetch_icon_bytes(app_id: &str) -> Result<Option<Vec<u8>>, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;

    let path = Path::new(app_id);
    if !path.is_file() {
        return Ok(None);
    }
    let wide: Vec<u16> = OsStr::new(app_id).encode_wide().chain(Some(0)).collect();

    unsafe {
        let mut info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            Default::default(),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if result == 0 || info.hIcon.0.is_null() {
            return Ok(None);
        }
        let hicon = info.hIcon;
        let png = windows_hicon_to_png(hicon).map_err(|e| format!("转换图标失败：{e}"))?;
        let _ = DestroyIcon(hicon);
        Ok(Some(png))
    }
}

#[cfg(target_os = "windows")]
unsafe fn windows_hicon_to_png(hicon: HICON) -> Result<Vec<u8>, String> {
    use std::io::Cursor;

    use image::ImageFormat;
    use windows::Win32::Graphics::Gdi::{
        DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

    let mut icon_info = ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_err() {
        return Err("GetIconInfo failed".into());
    }
    let hbm = icon_info.hbmColor;
    if hbm.0.is_null() {
        return Err("icon bitmap missing".into());
    }

    let mut bitmap = BITMAP::default();
    if GetObjectW(
        hbm,
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bitmap as *mut _ as *mut _),
    ) == 0
    {
        let _ = DeleteObject(icon_info.hbmColor);
        let _ = DeleteObject(icon_info.hbmMask);
        return Err("GetObjectW failed".into());
    }

    let width = bitmap.bmWidth.unsigned_abs() as u32;
    let height = bitmap.bmHeight.unsigned_abs() as u32;
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: bitmap.bmWidth,
            biHeight: -bitmap.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let hdc = GetDC(None);
    let lines = GetDIBits(
        hdc,
        hbm,
        0,
        height,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );
    let _ = ReleaseDC(None, hdc);
    let _ = DeleteObject(icon_info.hbmColor);
    let _ = DeleteObject(icon_info.hbmMask);

    if lines == 0 {
        return Err("GetDIBits failed".into());
    }

    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    let img = image::RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| "invalid icon dimensions".to_string())?;
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| format!("编码 PNG 失败：{e}"))?;
    Ok(buf)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_fetch_icon_bytes(app_id: &str) -> Result<Option<Vec<u8>>, String> {
    let wm_class = app_id.split(':').next().unwrap_or(app_id);
    let Some(icon_name) = linux_desktop_icon_name(wm_class) else {
        return Ok(None);
    };
    let Some(path) = linux_resolve_icon_path(&icon_name) else {
        return Ok(None);
    };
    let bytes = std::fs::read(&path).map_err(|e| format!("读取图标失败：{e}"))?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("解码图标失败：{e}"))?;
    let mut buf = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )
    .map_err(|e| format!("编码 PNG 失败：{e}"))?;
    Ok(Some(buf))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_desktop_icon_name(wm_class: &str) -> Option<String> {
    let class_lower = wm_class.to_ascii_lowercase();
    for dir in super::foreground::linux_desktop_dirs() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if !super::foreground::desktop_matches_wm_class(&content, &class_lower) {
                continue;
            }
            if let Some(icon) = super::foreground::desktop_field(&content, "Icon") {
                return Some(icon);
            }
        }
    }
    None
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_resolve_icon_path(icon_name: &str) -> Option<std::path::PathBuf> {
    if icon_name.starts_with('/') {
        let path = std::path::PathBuf::from(icon_name);
        return path.is_file().then_some(path);
    }

    let mut candidates = vec![
        std::path::PathBuf::from(format!("/usr/share/pixmaps/{icon_name}.png")),
        std::path::PathBuf::from(format!("/usr/share/pixmaps/{icon_name}.svg")),
    ];

    for size in ["512x512", "256x256", "128x128", "64x64", "48x48", "32x32"] {
        candidates.push(std::path::PathBuf::from(format!(
            "/usr/share/icons/hicolor/{size}/apps/{icon_name}.png"
        )));
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home = std::path::PathBuf::from(home);
        for size in ["512x512", "256x256", "128x128", "64x64", "48x48", "32x32"] {
            candidates.push(
                home.join(format!(
                    ".local/share/icons/hicolor/{size}/apps/{icon_name}.png"
                )),
            );
        }
    }

    candidates.into_iter().find(|p| p.is_file())
}
