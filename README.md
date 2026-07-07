# CUFE-COURSE Mobile

中央财经大学课程表移动端应用，基于 `Vue 3 + Tauri 2` 构建，支持 Android 打包。

## 当前版本

- `2.7.1`

## 本次更新重点

- 优化 Android 构建配置，仅编译 arm64-v8a ABI，减小 APK 体积
- APK 文件名明确标识 arm64-v8a 架构

## 常用命令

```bash
npm install
npm run build
npm run build:android
```

## Android Release 产物

- APK: `src-tauri/gen/android/app/build/outputs/apk/universal/release/cufe-course-v2.7.1-arm64-v8a-release.apk`
