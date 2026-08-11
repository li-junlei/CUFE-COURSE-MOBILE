# CUFE-COURSE Mobile

中央财经大学课程表移动端应用，基于 `Vue 3 + Tauri 2` 构建，支持 Android 打包。

## 当前版本

- `2.7.2`

## 本次更新重点

- 修复单双周课程（周次含 `(单)`/`(双)` 标记，如"应用计量经济学"）在课表中缺失的问题

## 常用命令

```bash
npm install
npm run build
npm run build:android
```

## Android Release 产物

- APK: `src-tauri/gen/android/app/build/outputs/apk/universal/release/cufe-course-v2.7.2-arm64-v8a-release.apk`
