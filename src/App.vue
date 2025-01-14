<template>
  <div id="root">
    <div class="header">
      <h1>坐标转换应用</h1>
    </div>
    <div class="row">
      <div class="select">
        <el-select v-model="sourceCRS" placeholder="选择源坐标系">
          <el-option
            v-for="option in options"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
        <div class="svg">
          <svg
            t="1736821848019"
            class="icon"
            viewBox="0 0 1024 1024"
            version="1.1"
            xmlns="http://www.w3.org/2000/svg"
            p-id="6967"
            width="22"
            height="32"
          >
            <path
              d="M950.857143 665.6H73.142857a43.885714 43.885714 0 0 1 0-87.771429h789.942857L696.32 355.474286a43.885714 43.885714 0 0 1 70.217143-52.662857l219.428571 292.571428a43.885714 43.885714 0 0 1-35.108571 70.217143z"
              p-id="6968"
            ></path>
          </svg>
        </div>
        <el-select v-model="targetCRS" placeholder="选择目标坐标系">
          <el-option
            v-for="option in options"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>
      <div class="point">
        <el-input
          v-model="point"
          style="width: 200px"
          placeholder="输入经度,纬度。例如:116.0,39.0"
        />
        <div class="svg">
          <svg
            t="1736821848019"
            class="icon"
            viewBox="0 0 1024 1024"
            version="1.1"
            xmlns="http://www.w3.org/2000/svg"
            p-id="6967"
            width="22"
            height="32"
          >
            <path
              d="M950.857143 665.6H73.142857a43.885714 43.885714 0 0 1 0-87.771429h789.942857L696.32 355.474286a43.885714 43.885714 0 0 1 70.217143-52.662857l219.428571 292.571428a43.885714 43.885714 0 0 1-35.108571 70.217143z"
              p-id="6968"
            ></path>
          </svg>
        </div>
        <el-input
          :value="pointResult"
          style="width: 200px; margin-right: 12px"
          placeholder="Please input"
          disabled
        />
        <el-button type="primary" @click="convertPoint">转换</el-button>
      </div>
    </div>

    <div class="upload">
      <el-button type="primary" @click="openFileDialog">选择文件</el-button>
      <el-button type="primary" @click="openFolderDialog"
        >打开文件所在位置</el-button
      >
      <el-button type="primary" @click="convertFiles">转换文件</el-button>
      <div class="fileHeader">
        <el-button type="primary" @click="selectSaveFolder"
          >选择保存文件夹</el-button
        >
        <div class="saveFolder">{{ saveFolder }}</div>
      </div>
    </div>
    <div class="result">
      <el-table :data="fileList">
        <el-table-column prop="name" label="文件名" />
        <el-table-column prop="status" label="状态" />
      </el-table>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { appDataDir, join, basename } from '@tauri-apps/api/path';
import { readTextFile } from '@tauri-apps/plugin-fs';
const point = ref('');
const pointResult = ref('');
const openFileDialog = async () => {
  const files = await open({
    multiple: true,
    directory: false,
    filters: [
      {
        name: 'geojson',
        extensions: ['geojson', 'json'],
      },
    ],
  });
  fileList.value = [];
  files.forEach(async (i) => {
    const name = await basename(i);
    fileList.value.push({
      name,
      path: i,
      status: '未转换',
    });
  });
};
// 转换单个点
const convertPoint = async () => {
  if (sourceCRS.value === targetCRS.value || !point.value) {
    return;
  }
  if (sourceCRS.value === 'WGS84') {
    let [lon, lat] = point.value.split(',');
    let result = await invoke('wgs84_to_gcj02', {
      lon: Number(lon),
      lat: Number(lat),
    });
    pointResult.value = `${result[0]},${result[1]}`;
  } else if (sourceCRS.value === 'GCJ02') {
    let [lon, lat] = point.value.split(',');
    let result = await invoke('gcj02_to_wgs84', {
      lon: Number(lon),
      lat: Number(lat),
    });
    pointResult.value = `${result[0]},${result[1]}`;
  }
};
// 选择保存文件夹
const saveFolder = ref('');
const selectSaveFolder = async () => {
  const path = await open({
    multiple: false,
    directory: true,
  });
  setSaveFolder(path);
};
// 选择文件夹
const openFolderDialog = async () => {
  if (!fileList.value.length) return;
  await invoke('open_file_location', {
    path: fileList.value[0].path,
  });
};
const sourceCRS = ref('WGS84');
const targetCRS = ref('GCJ02');
const options = ref([
  { label: 'WGS84', value: 'WGS84' },
  { label: 'GCJ02', value: 'GCJ02' },
]);
const fileList = ref([]); //文件列表
// 转换文件
const convertFiles = async () => {
  if (!isCoordinate) return;
  const time = Date.now().toString();
  fileList.value.forEach(async (file) => {
    const fileName = await basename(file.path); //文件名
    const path = await join(saveFolder.value, time, fileName); //保存路径
    const result = await invoke('convert_geojson_coordinates', {
      input: file.path,
      output: path,
      source: sourceCRS.value,
      target: targetCRS.value,
    });
    file.status = '已转换';
    file.path = path;
  });
};
//是否可以转换
function isCoordinate() {
  if (fileList.value.length === 0) {
    return false;
  }
  if (sourceCRS.value === targetCRS.value) {
    return false;
  }
  return true;
}
async function setSaveFolder(val) {
  if (!val) {
    val = await appDataDir();
  }
  saveFolder.value = val;
  localStorage.setItem('saveFolder', saveFolder.value);
}
onMounted(() => {
  let saveFolderLocal = localStorage.getItem('saveFolder');
  if (saveFolderLocal) {
    saveFolder.value = saveFolderLocal;
  } else {
    setSaveFolder();
  }
});
</script>

<style>
#root {
  text-align: center;
  font-family: Arial, sans-serif;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.fileHeader {
  flex: 1;
  display: flex;
  align-items: center;
  margin-left: 12px;
}
.saveFolder {
  max-width: 500px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-left: 12px;
}
.point {
  display: flex;
  align-items: center;
}
.svg {
  margin: 0 12px;
}
.result {
  width: 100%;
  flex: 1;
  overflow: auto;
  margin-top: 12px;
}

.row {
  width: 100%;
  display: flex;
  flex-direction: row;
  align-items: center;
  margin-bottom: 20px;
}
.row > .select {
  margin-right: 12px;
  display: flex;
  align-items: center;
  width: 250px;
  justify-content: space-around;
}
.upload {
  display: flex;
  align-items: center;
  width: 100%;
}
</style>
