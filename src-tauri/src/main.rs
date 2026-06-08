#[derive(serde::Serialize)]
struct DesktopHealth {
    status: &'static str,
    product_name: &'static str,
    version: &'static str,
}

#[tauri::command]
fn desktop_health() -> DesktopHealth {
    DesktopHealth {
        status: "ok",
        product_name: "Time State Recorder Desktop",
        version: env!("CARGO_PKG_VERSION"),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![desktop_health])
        .run(tauri::generate_context!())
        .expect("error while running Time State Recorder desktop");
}
