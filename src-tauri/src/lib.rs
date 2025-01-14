use serde_json::Value as JsonValue;
use std::f64::consts::PI;
use tauri::command;
use std::path::PathBuf;
use std::fs;  // 添加这一行

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
fn get_conversion_method(source: &str, target: &str) -> Option<fn(f64, f64) -> (f64, f64)> {
    match (source, target) {
        ("WGS84", "GCJ02") => Some(wgs84_to_gcj02),
        ("GCJ02", "WGS84") => Some(gcj02_to_wgs84),
        _ => None,
    }
}

fn process_coordinates(
    coordinates: &mut JsonValue,
    conversion_fn: fn(f64, f64) -> (f64, f64),
    geom_type: &str,
) {
    match geom_type {
        "Point" => {
            if let Some(coord) = coordinates.as_array_mut() {
                if let (Some(lon), Some(lat)) = (coord.get(0), coord.get(1)) {
                    let (new_lon, new_lat) =
                        conversion_fn(lon.as_f64().unwrap(), lat.as_f64().unwrap());
                    coord[0] = JsonValue::from(new_lon);
                    coord[1] = JsonValue::from(new_lat);
                }
            }
        }
        "MultiPoint" | "LineString" => {
            if let Some(coords) = coordinates.as_array_mut() {
                for coord in coords {
                    process_coordinates(coord, conversion_fn, "Point");
                }
            }
        }
        "Polygon" | "MultiLineString" => {
            if let Some(rings) = coordinates.as_array_mut() {
                for ring in rings {
                    process_coordinates(ring, conversion_fn, "LineString");
                }
            }
        }
        "MultiPolygon" => {
            if let Some(polygons) = coordinates.as_array_mut() {
                for polygon in polygons {
                    process_coordinates(polygon, conversion_fn, "Polygon");
                }
            }
        }
        _ => {}
    }
}

fn process_geometry(geometry: &mut JsonValue, conversion_fn: fn(f64, f64) -> (f64, f64)) {
    if let Some(geom_type) = geometry
        .get("type")
        .and_then(|t| t.as_str())
        .map(String::from)
    {
        if let Some(coordinates) = geometry.get_mut("coordinates") {
            process_coordinates(coordinates, conversion_fn, &geom_type);
        }
    }
}

fn process_feature(feature: &mut JsonValue, conversion_fn: fn(f64, f64) -> (f64, f64)) {
    if let Some(geometry) = feature.get_mut("geometry") {
        process_geometry(geometry, conversion_fn);
    }
}

#[tauri::command]
fn convert_geojson_coordinates(input: String,output: String, source: &str, target: &str) -> Result<String, String> {
     // 读取输入文件
    let content = fs::read_to_string(&input)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 解析 JSON
    let mut geojson: JsonValue = serde_json::from_str(&content)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;
    if let Some(conversion_fn) = get_conversion_method(source, target) {
        if let Some(geojson_type) = geojson.get("type").and_then(|t| t.as_str()) {
            match geojson_type {
                "FeatureCollection" => {
                    if let Some(features) =
                        geojson.get_mut("features").and_then(|f| f.as_array_mut())
                    {
                        for feature in features {
                            process_feature(feature, conversion_fn);
                        }
                    }
                }
                "Feature" => {
                    process_feature(&mut geojson, conversion_fn);
                }
                "GeometryCollection" => {
                    if let Some(geometries) =
                        geojson.get_mut("geometries").and_then(|g| g.as_array_mut())
                    {
                        for geometry in geometries {
                            process_geometry(geometry, conversion_fn);
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        eprintln!("Unsupported conversion from {} to {}", source, target);
    }
     // 写入转换后的文件
    let converted_json = serde_json::to_string(&geojson)
        .map_err(|e| format!("序列化 JSON 失败: {}", e))?;
    
    write_file_with_path(output.clone(), converted_json)?;
    Ok(output)
}
#[tauri::command]
fn write_file_with_path(path: String, content: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    // 写入文件
    std::fs::write(&path, content)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(path.to_string_lossy().into_owned())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            gcj02_to_wgs84,
            wgs84_to_gcj02,
            convert_geojson_coordinates,
            write_file_with_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
