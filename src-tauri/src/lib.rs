use std::f64::consts::PI;
use tauri::command;

/// 判断是否在中国范围内
fn out_of_china(lon: f64, lat: f64) -> bool {
    !(lon > 73.66 && lon < 135.05 && lat > 3.86 && lat < 53.55)
}

/// 转换纬度
fn transform_lat(x: f64, y: f64) -> f64 {
    let mut ret = -100.0 + 2.0 * x + 3.0 * y + 0.2 * y * y + 0.1 * x * y + 0.2 * (x.abs().sqrt());
    ret += (20.0 * (6.0 * x * PI).sin() + 20.0 * (2.0 * x * PI).sin()) * 2.0 / 3.0;
    ret += (20.0 * (y * PI).sin() + 40.0 * (y / 3.0 * PI).sin()) * 2.0 / 3.0;
    ret += (160.0 * (y / 12.0 * PI).sin() + 320.0 * (y * PI / 30.0).sin()) * 2.0 / 3.0;
    ret
}

/// 转换经度
fn transform_lon(x: f64, y: f64) -> f64 {
    let mut ret = 300.0 + x + 2.0 * y + 0.1 * x * x + 0.1 * x * y + 0.1 * (x.abs().sqrt());
    ret += (20.0 * (6.0 * x * PI).sin() + 20.0 * (2.0 * x * PI).sin()) * 2.0 / 3.0;
    ret += (20.0 * (x * PI).sin() + 40.0 * (x / 3.0 * PI).sin()) * 2.0 / 3.0;
    ret += (150.0 * (x / 12.0 * PI).sin() + 300.0 * (x * PI / 30.0).sin()) * 2.0 / 3.0;
    ret
}

//  WGS84 转 GCJ02
#[command]
fn wgs84_to_gcj02(lon: f64, lat: f64) -> (f64, f64) {
    if out_of_china(lon, lat) {
        return (lon, lat); // 不在中国范围内，直接返回
    }

    let a = 6378245.0; // 地球半径
    let ee = 0.00669342162296594323; // 偏心率平方

    let d_lat = transform_lat(lon - 105.0, lat - 35.0);
    let d_lon = transform_lon(lon - 105.0, lat - 35.0);
    let rad_lat = lat / 180.0 * PI;
    let magic = 1.0 - ee * (rad_lat.sin() * rad_lat.sin());
    let sqrt_magic = magic.sqrt();

    let mg_lat = lat + (d_lat * 180.0) / ((a * (1.0 - ee)) / (magic * sqrt_magic) * PI);
    let mg_lon = lon + (d_lon * 180.0) / (a / sqrt_magic * rad_lat.cos() * PI);

    (mg_lon, mg_lat) // 经度在前，纬度在后
}

// GCJ02 转 WGS84
#[command]
fn gcj02_to_wgs84(lon: f64, lat: f64) -> (f64, f64) {
    if out_of_china(lon, lat) {
        return (lon, lat); // 不在中国范围内，直接返回
    }

    let (mut lon_tmp, mut lat_tmp) = (lon, lat);
    let (mut d_lat, mut d_lon);

    loop {
        let (mg_lon, mg_lat) = wgs84_to_gcj02(lon_tmp, lat_tmp);
        d_lat = mg_lat - lat;
        d_lon = mg_lon - lon;

        // 提高精度：设置误差阈值为 1e-8
        if d_lat.abs() < 1e-8 && d_lon.abs() < 1e-8 {
            break;
        }

        lat_tmp -= d_lat;
        lon_tmp -= d_lon;
    }

    (lon_tmp, lat_tmp) // 经度在前，纬度在后
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![gcj02_to_wgs84, wgs84_to_gcj02])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
