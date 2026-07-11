use tauri::http::{header, Request, Response, StatusCode};
use tauri::{Manager, UriSchemeContext};

use super::app_icon;
use super::db::{blob_path, thumb_path};
use super::state::ClipboardAssistantState;

pub fn register_protocols<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .register_uri_scheme_protocol("clipboard-app-icon", |ctx, request| {
            handle_app_icon(ctx, request)
        })
        .register_uri_scheme_protocol("clipboard-thumb", |ctx, request| {
            handle_thumb(ctx, request)
        })
}

fn handle_app_icon<R: tauri::Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let app = ctx.app_handle();
    let Some(bundle) = parse_path_segment(&request).filter(|b| !b.is_empty()) else {
        return text_response(StatusCode::BAD_REQUEST, "missing bundle id");
    };

    if let Ok(path) = app_icon::ensure_cached(app, &bundle) {
        if let Ok(data) = std::fs::read(path) {
            return png_response(data);
        }
    }
    if let Some(data) = app_icon::read_cached(app, &bundle) {
        return png_response(data);
    }
    text_response(StatusCode::NOT_FOUND, "icon not found")
}

fn handle_thumb<R: tauri::Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let app = ctx.app_handle();
    let id_str = parse_path_segment(&request).unwrap_or_default();
    let Ok(id) = id_str.parse::<i64>() else {
        return text_response(StatusCode::BAD_REQUEST, "invalid id");
    };

    let Some(state) = app.try_state::<std::sync::Arc<ClipboardAssistantState>>() else {
        return text_response(StatusCode::SERVICE_UNAVAILABLE, "state unavailable");
    };
    let thumb = thumb_path(&state.blobs_dir, id);
    if thumb.is_file() {
        if let Ok(data) = std::fs::read(&thumb) {
            return webp_response(data);
        }
    }
    let blob = blob_path(&state.blobs_dir, id);
    if blob.is_file() {
        if let Ok(data) = std::fs::read(&blob) {
            return octet_response(data);
        }
    }
    text_response(StatusCode::NOT_FOUND, "thumb not found")
}

fn parse_path_segment(request: &Request<Vec<u8>>) -> Option<String> {
    let path = request.uri().path().trim_start_matches('/');
    if path.is_empty() {
        return request.uri().host().map(str::to_string);
    }
    Some(urldecode(path))
}

fn urldecode(raw: &str) -> String {
    let mut out = String::new();
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(v as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn png_response(data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(data)
        .unwrap_or_else(|_| text_response(StatusCode::INTERNAL_SERVER_ERROR, "response error"))
}

fn webp_response(data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/webp")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(data)
        .unwrap_or_else(|_| text_response(StatusCode::INTERNAL_SERVER_ERROR, "response error"))
}

fn octet_response(data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(data)
        .unwrap_or_else(|_| text_response(StatusCode::INTERNAL_SERVER_ERROR, "response error"))
}

fn text_response(status: StatusCode, message: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(message.as_bytes().to_vec())
        .unwrap()
}
