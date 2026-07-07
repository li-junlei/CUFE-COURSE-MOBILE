# Changelog

## 2.7.1

- 优化 Android 构建配置，仅编译 arm64-v8a ABI，减小 APK 体积
- 将 APK 重命名为 `cufe-course-v{version}-arm64-v8a-release.apk`，方便识别架构

## 2.7.0

- 修复 Android 小部件数据已同步但桌面未即时刷新的问题
- 修复小部件跨天、跨周后仍显示旧快照的问题
- 修复小部件课程周次判断错误，避免当天有课时误显示"课程结束啦"
- 统一前端、Rust、Android 三端的小部件同步链路
- 更新 Android release 构建产物版本号到 `2.7.0`
