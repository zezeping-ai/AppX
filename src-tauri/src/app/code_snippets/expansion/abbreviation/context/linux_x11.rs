//! Linux X11 + AT-SPI2：读取焦点控件光标前文本。

use zbus::blocking::connection::Builder;
use zbus::blocking::Connection;
use zbus::zvariant::OwnedObjectPath;

const A11Y_BUS: &str = "org.a11y.Bus";
const ATSPI_REGISTRY: &str = "org.a11y.atspi.Registry";
const ATSPI_ROOT: &str = "/org/a11y/atspi/accessible/root";

pub fn read_text_before_cursor(max_len: usize) -> Option<String> {
    if max_len == 0 {
        return Some(String::new());
    }

    let a11y = a11y_connection()?;
    let (bus_name, path) = get_focused_accessible(&a11y)?;
    let caret = get_caret_offset(&a11y, &bus_name, &path)?;
    if caret <= 0 {
        return Some(String::new());
    }
    let start = caret.saturating_sub(max_len as i32);
    let text = get_text(&a11y, &bus_name, &path, start, caret)?;
    Some(take_suffix_chars(&text, max_len))
}

fn a11y_connection() -> Option<Connection> {
    if let Ok(addr) = std::env::var("AT_SPI_BUS_ADDRESS") {
        if !addr.is_empty() {
            if let Ok(conn) = Builder::address(addr.as_str()).and_then(|b| b.build()) {
                return Some(conn);
            }
        }
    }

    let session = Connection::session().ok()?;
    let addr = session
        .call_method(
            Some(A11Y_BUS),
            "/org/a11y/bus",
            Some("org.a11y.Bus"),
            "GetAddress",
            &(),
        )
        .ok()?
        .body()
        .deserialize::<String>()
        .ok()?;
    Builder::address(addr.as_str()).ok()?.build().ok()
}

fn get_focused_accessible(conn: &Connection) -> Option<(String, String)> {
    let reply = conn
        .call_method(
            Some(ATSPI_REGISTRY),
            ATSPI_ROOT,
            Some("org.a11y.atspi.Accessible"),
            "GetFocus",
            &(),
        )
        .ok()?;
    let body = reply.body();
    let (bus_name, path): (String, OwnedObjectPath) = body.deserialize().ok()?;
    Some((bus_name, path.to_string()))
}

fn get_caret_offset(conn: &Connection, bus_name: &str, path: &str) -> Option<i32> {
    let reply = conn
        .call_method(
            Some(bus_name),
            path,
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.a11y.atspi.Text", "CaretOffset"),
        )
        .ok()?;
    let body = reply.body();
    let value: zbus::zvariant::Value<'_> = body.deserialize().ok()?;
    i32::try_from(value).ok()
}

fn get_text(conn: &Connection, bus_name: &str, path: &str, start: i32, end: i32) -> Option<String> {
    let reply = conn
        .call_method(
            Some(bus_name),
            path,
            Some("org.a11y.atspi.Text"),
            "GetText",
            &(start, end),
        )
        .ok()?;
    reply.body().deserialize::<String>().ok()
}

fn take_suffix_chars(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    text.chars()
        .skip(text.chars().count().saturating_sub(max_len))
        .collect()
}
