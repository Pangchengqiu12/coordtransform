<template>
  <div id="root">
    <h1>坐标转换应用</h1>
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
        <el-select v-model="targetCRS" placeholder="选择目标坐标系">
          <el-option
            v-for="option in options"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </div>
      <div class="upload">
        <el-upload
          class="upload-demo"
          drag
          multiple
          :on-change="handleFileChange"
          :on-remove="handleRemove"
          :auto-upload="false"
          :show-file-list="false"
        >
          <i class="el-icon-upload"></i>
          <div class="el-upload__text">将文件拖到此处，或<em>点击上传</em></div>
        </el-upload>
      </div>
    </div>
    <el-button type="primary" @click="convertFiles" style="width: 100%"
      >转换文件</el-button
    >
    <div class="result">
      <el-table :data="fileList">
        <el-table-column prop="name" label="文件名" />
        <el-table-column prop="status" label="状态" />
      </el-table>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const sourceCRS = ref('WGS84');
const targetCRS = ref('GCJ02');
const options = ref([
  { label: 'WGS84', value: 'WGS84' },
  { label: 'GCJ02', value: 'GCJ02' },
]);
const fileList = ref([]);

const uploadUrl = '/upload'; // 假设的上传 URL

const handleFileChange = (file, files) => {
  // 更新文件列表
  fileList.value = files;
  console.log(fileList.value);
};

const handleRemove = (file, fileList) => {
  // 处理文件移除
  fileList.value = fileList;
  console.log(fileList);
};

const convertFiles = async () => {
  for (const file of fileList.value) {
    const result = await invoke('convert_coordinates', {
      file: file.raw,
      sourceCRS: sourceCRS.value,
      targetCRS: targetCRS.value,
    });
    file.status = result.success ? '转换成功' : '转换失败';
  }
};
</script>

<style>
#root {
  text-align: center;
  font-family: Arial, sans-serif;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.result {
  width: 100%;
  flex: 1;
  overflow: auto;
  margin-top: 12px;
}
.upload-demo {
  margin: 20px 0;
}
.row {
  width: 100%;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
}
.row > .select {
  flex: 1;
  margin-right: 20px;
  display: flex;
  flex-direction: column;
  height: 101px;
  justify-content: space-around;
}
.row > .upload {
  flex: 3;
}
</style>
