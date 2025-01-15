export interface FileList {
  name: string;
  path: string;
  status: 0 | 1 | 2 | 3; //0未转换 1已转换 2转换失败 3转换中;
}
