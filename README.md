# CUFE-COURSE Mobile

中央财经大学课程表移动端应用，基于 `Vue 3 + Tauri 2` 构建，支持 Android 打包。

## 当前版本

- `2.7.0`

## 本次更新重点

- 修复 Android 小部件在课表变化后无法及时刷新的问题
- 修复小部件跨天后仍读取旧课程快照的问题
- 修复小部件周次筛选逻辑，兼容项目当前使用的展开周次数组格式
- 优化小部件定时刷新兼容性，在无法使用精确定时闹钟时自动降级

## 常用命令

```bash
npm install
npm run build
npm run build:android
```

## Android Release 产物

- APK: `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`
- AAB: `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab`
