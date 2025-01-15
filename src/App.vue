<template>
  <div id="root">
    <div class="header">
      <a
        href="https://github.com/Pangchengqiu12/coordtransform"
        target="_blank"
      >
        <img src="/logo.png" class="logo vite" alt="Vite logo" />
      </a>
      <h1>坐标系转换</h1>
    </div>
    <div class="row box">
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
            width="20"
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
          style="width: 300px"
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
            width="20"
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
          style="width: 300px; margin-right: 12px"
          placeholder=""
          disabled
        />
        <el-button type="primary" @click="convertPoint">转换</el-button>
      </div>
    </div>

    <div class="result box">
      <div class="upload">
        <el-button
          type="primary"
          plain
          @click="openFileDialog"
          :disabled="isRunning"
          >选择文件</el-button
        >

        <el-button type="success" :disabled="isRunning" @click="convertFiles"
          >转换文件</el-button
        >
        <el-button type="primary" @click="openFolderDialog"
          >打开文件位置</el-button
        >
        <div class="fileHeader">
          <el-button
            type="primary"
            :disabled="isRunning"
            @click="selectSaveFolder"
            >选择保存文件夹</el-button
          >
          <el-text class="text">{{ saveFolder }}</el-text>
        </div>
      </div>
      <el-table
        :data="fileList"
        :cell-style="{ textAlign: 'center' }"
        :header-cell-style="{ 'text-align': 'center' }"
        height="90%"
      >
        <el-table-column prop="name" label="文件名" />
        <el-table-column prop="status" label="状态" width="200">
          <template #default="scope">
            <el-tag v-if="scope.row.status === 0" type="primary">未转换</el-tag>
            <el-tag v-else-if="scope.row.status === 1" type="success"
              >已转换</el-tag
            >
            <el-tag v-else-if="scope.row.status === 2" type="danger"
              >转换失败</el-tag
            >
            <el-tag v-else-if="scope.row.status === 3" type="warning"
              >转换中</el-tag
            >
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir, join, basename } from "@tauri-apps/api/path";
import type { FileList } from "./index";
const point = ref("");
const pointResult = ref("");
// 选择需要转换的文件
const openFileDialog = async () => {
  const files = await open({
    multiple: true,
    directory: false,
    filters: [
      {
        name: "geojson",
        extensions: ["geojson", "json"],
      },
    ],
  });
  if (files) {
    fileList.value = [];
    files.forEach(async (i) => {
      const name = await basename(i);
      fileList.value.push({
        name,
        path: i,
        status: 0,
      });
    });
  }
};
// 转换单个点
const convertPoint = async () => {
  if (sourceCRS.value === targetCRS.value || !point.value) {
    ElMessage({
      message: "源坐标系和目标坐标系不能相同，且坐标不能为空",
      type: "warning",
    });
    return;
  }
  if (sourceCRS.value === "WGS84") {
    let [lon, lat] = point.value.split(",");
    let result = (await invoke("wgs84_to_gcj02", {
      lon: Number(lon),
      lat: Number(lat),
    })) as [number, number];
    pointResult.value = `${result[0]},${result[1]}`;
  } else if (sourceCRS.value === "GCJ02") {
    let [lon, lat] = point.value.split(",");
    let result = (await invoke("gcj02_to_wgs84", {
      lon: Number(lon),
      lat: Number(lat),
    })) as [number, number];
    pointResult.value = `${result[0]},${result[1]}`;
  }
};
// 选择保存文件夹
const saveFolder = ref("");
const selectSaveFolder = async () => {
  const path = (await open({
    multiple: false,
    directory: true,
  })) as string;
  path && setSaveFolder(path);
};
// 打开文件位置
const openFolderDialog = async () => {
  if (!fileList.value.length) {
    ElMessage({
      message: "请先选择文件",
      type: "warning",
    });
    return;
  }
  await invoke("open_file_location", {
    path: fileList.value[0].path,
  });
};
const sourceCRS = ref("WGS84");
const targetCRS = ref("GCJ02");
const options = ref([
  { label: "WGS84", value: "WGS84" },
  { label: "GCJ02", value: "GCJ02" },
]);
const isRunning = computed(() => {
  return fileList.value.length && fileList.value.some((i) => i.status === 3);
});
const isDone = computed(() => {
  return fileList.value.length && !fileList.value.every((i) => i.status === 0);
});

const fileList = ref<FileList[]>([]); //文件列表
// 转换文件
const convertFiles = async () => {
  if (!isCoordinate()) return;
  const time = Date.now().toString();
  fileList.value.forEach(async (file) => {
    file.status = 3;
    try {
      const fileName = await basename(file.path); //文件名
      const path = await join(saveFolder.value, time, fileName); //保存路径
      await invoke("convert_geojson_coordinates", {
        input: file.path,
        output: path,
        source: sourceCRS.value,
        target: targetCRS.value,
      });
      file.status = 1;
      file.path = path;
    } catch (error) {
      file.status = 2;
    }
  });
};
//是否可以转换
function isCoordinate() {
  if (fileList.value.length === 0) {
    ElMessage({
      message: "请先选择文件",
      type: "warning",
    });
    return false;
  } else if (isRunning.value) {
    ElMessage({
      message: "文件转换中",
      type: "warning",
    });
    return false;
  } else if (sourceCRS.value === targetCRS.value) {
    ElMessage({
      message: "源坐标系和目标坐标系不能相同",
      type: "warning",
    });
    return false;
  } else if (isDone.value) {
    ElMessage({
      message: "文件已全部转换",
      type: "warning",
    });
    return false;
  }

  return true;
}

// 设置保存文件夹
async function setSaveFolder(val?: string) {
  if (!val) {
    val = await appDataDir();
  }
  saveFolder.value = val;
  localStorage.setItem("saveFolder", saveFolder.value);
}
onMounted(() => {
  let saveFolderLocal = localStorage.getItem("saveFolder");
  if (saveFolderLocal) {
    saveFolder.value = saveFolderLocal;
  } else {
    setSaveFolder();
  }
});
</script>

<style>
#root {
  position: relative;
  text-align: center;
  font-family: Arial, sans-serif;
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 97.4vh;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
  overflow: hidden;
}
.text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
  text-indent: 12px;
}
a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}
.logo {
  height: 5em;
  padding: 1em;
  will-change: filter;
  transition: 0.75s;
}
.logo.vite:hover {
  filter: drop-shadow(0 0 1em #747bff);
}
.header {
  display: flex;
  align-items: center;
  width: 100%;
  justify-content: center;
}
h1 {
  color: #333;
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
  width: 100%;
  display: flex;
  align-items: center;
}
.svg {
  margin: 0 12px;
}
.result {
  box-sizing: border-box;
  background-color: #fff;
  width: 99%;
  flex: 1;
  overflow: auto;
  margin-top: 6px;
}
.box {
  border-radius: 4px;
  padding: 6px 12px;
  overflow: hidden;
  box-shadow: 0 -3px 6px -2px rgba(0, 0, 0, 0.1),
    0 3px 6px -2px rgba(0, 0, 0, 0.1), -3px 0 6px -2px rgba(0, 0, 0, 0.1),
    3px 0 6px -2px rgba(0, 0, 0, 0.1);
}
.row {
  box-sizing: border-box;
  width: 99%;
  background-color: #fff;
}
.row > .select {
  margin-right: 12px;
  display: flex;
  align-items: center;
  width: 644px;
  justify-content: space-around;
}
.upload {
  display: flex;
  align-items: center;
  width: 100%;
}
</style>
