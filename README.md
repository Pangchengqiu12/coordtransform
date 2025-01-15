# 坐标系转换  Tauri + Rust + Vue + TypeScript

目前主要是WGS84和GCJ02(高德)坐标的相互转换，支持单个坐标点转换，geojson格式的文件转换和批量转换。转换逻辑使用rust编写，实现快速转化.测试14个100KB的文件转换1秒不到。

支持**Apache License 2.0** 开源协议
