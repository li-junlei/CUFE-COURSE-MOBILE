package com.lijunlei.cufecourse

import android.app.AlarmManager
import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log
import android.widget.RemoteViews
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.Calendar

/**
 * 课程表桌面小组件 Provider
 * 2x2 规格，显示当天尚未结束的最新两门课程
 */
class CourseWidgetProvider : AppWidgetProvider() {

    companion object {
        const val ACTION_UPDATE = "com.lijunlei.cufecourse.ACTION_WIDGET_UPDATE"
        private const val TAG = "CourseWidget"
        private const val FALLBACK_INTERVAL_MS = 30 * 60 * 1000L

        private data class RenderResult(
            val nextTriggerAt: Long,
        )

        private data class CourseSnapshot(
            val name: String,
            val location: String,
            val startTime: String,
            val endTime: String,
        )

        // Widget 数据文件路径
        private fun getDataFile(context: Context): File {
            val dataDir = File("/data/data/${context.packageName}/files/widget")
            if (!dataDir.exists()) {
                dataDir.mkdirs()
            }
            return File(dataDir, "widget_data.json")
        }

        /**
         * 更新 Widget
         */
        fun updateWidget(context: Context, appWidgetManager: AppWidgetManager, appWidgetId: Int) {
            try {
                val views = RemoteViews(context.packageName, R.layout.widget_course_2x2)

                bindClickToLaunchMain(context, views)

                val nextTriggerAt = try {
                    val dataFile = getDataFile(context)
                    if (dataFile.exists()) {
                        val json = JSONObject(dataFile.readText())
                        updateViewsFromJson(views, json).nextTriggerAt
                    } else {
                        showDefaultState(views)
                        fallbackTriggerTime()
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Failed to read widget data", e)
                    showDefaultState(views)
                    fallbackTriggerTime()
                }

                appWidgetManager.updateAppWidget(appWidgetId, views)
                scheduleUpdate(context, nextTriggerAt)
            } catch (e: Exception) {
                Log.e(TAG, "updateWidget failed", e)
            }
        }

        private fun bindClickToLaunchMain(context: Context, views: RemoteViews) {
            var launchIntent: Intent? = null
            try {
                launchIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)
            } catch (e: Exception) {
                Log.e(TAG, "getLaunchIntentForPackage failed", e)
            }

            if (launchIntent == null) {
                try {
                    launchIntent = Intent(context, Class.forName("com.lijunlei.cufecourse.MainActivity"))
                } catch (e: Exception) {
                    Log.e(TAG, "MainActivity not found", e)
                }
            }

            if (launchIntent != null) {
                launchIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                launchIntent.addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
                val pendingIntent = PendingIntent.getActivity(
                    context,
                    0,
                    launchIntent,
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
                )
                views.setOnClickPendingIntent(R.id.widget_container, pendingIntent)
            }
        }

        /**
         * 从 JSON 数据更新视图，并按当前系统时间动态筛选当天未结束课程。
         */
        private fun updateViewsFromJson(views: RemoteViews, json: JSONObject): RenderResult {
            val calendar = Calendar.getInstance()
            val nowMinutes = calendar.get(Calendar.HOUR_OF_DAY) * 60 + calendar.get(Calendar.MINUTE)

            return try {
                // 顶部状态栏：周次取数据源，日期星期取系统当前时间
                val currentWeek = json.optInt("currentWeek", 1)
                val currentDate = "${calendar.get(Calendar.MONTH) + 1}月${calendar.get(Calendar.DAY_OF_MONTH)}日"
                val dayName = getDayName(calendar.get(Calendar.DAY_OF_WEEK))
                views.setTextViewText(R.id.widget_header, "第 ${currentWeek} 周  ${currentDate}  ${dayName}")

                val courses = json.optJSONArray("courses") ?: JSONArray()
                val notEndedCourses = mutableListOf<CourseSnapshot>()
                var nextCourseEndMinutes: Int? = null

                for (i in 0 until courses.length()) {
                    val course = courses.optJSONObject(i) ?: continue
                    val endTime = course.optString("endTime", "")
                    val endMinutes = toMinutes(endTime)
                    if (endMinutes != null && endMinutes > nowMinutes) {
                        notEndedCourses.add(
                            CourseSnapshot(
                                name = course.optString("name", ""),
                                location = course.optString("location", ""),
                                startTime = course.optString("startTime", ""),
                                endTime = endTime,
                            ),
                        )
                        if (nextCourseEndMinutes == null || endMinutes < nextCourseEndMinutes) {
                            nextCourseEndMinutes = endMinutes
                        }
                    }
                }

                notEndedCourses.sortBy { toMinutes(it.startTime) ?: Int.MAX_VALUE }
                val displayCourses = notEndedCourses.take(2)

                if (displayCourses.isEmpty()) {
                    views.setViewVisibility(R.id.courses_container, android.view.View.GONE)
                    views.setViewVisibility(R.id.empty_message, android.view.View.VISIBLE)
                    views.setTextViewText(R.id.empty_message, "课程结束啦 🎉")
                } else {
                    views.setViewVisibility(R.id.courses_container, android.view.View.VISIBLE)
                    views.setViewVisibility(R.id.empty_message, android.view.View.GONE)

                    val course1 = displayCourses[0]
                    views.setTextViewText(R.id.course1_name, course1.name)
                    views.setTextViewText(R.id.course1_location, course1.location)
                    views.setTextViewText(R.id.course1_time, "${course1.startTime} - ${course1.endTime}")
                    views.setViewVisibility(R.id.course1_layout, android.view.View.VISIBLE)

                    if (displayCourses.size > 1) {
                        val course2 = displayCourses[1]
                        views.setTextViewText(R.id.course2_name, course2.name)
                        views.setTextViewText(R.id.course2_location, course2.location)
                        views.setTextViewText(R.id.course2_time, "${course2.startTime} - ${course2.endTime}")
                        views.setViewVisibility(R.id.course2_layout, android.view.View.VISIBLE)
                        views.setViewVisibility(R.id.divider, android.view.View.VISIBLE)
                    } else {
                        views.setViewVisibility(R.id.course2_layout, android.view.View.GONE)
                        views.setViewVisibility(R.id.divider, android.view.View.GONE)
                    }
                }

                RenderResult(nextTriggerAt = calculateNextTriggerTime(calendar, nextCourseEndMinutes))
            } catch (e: Exception) {
                Log.e(TAG, "Failed to parse JSON", e)
                showDefaultState(views)
                RenderResult(nextTriggerAt = fallbackTriggerTime())
            }
        }

        private fun getDayName(dayOfWeek: Int): String {
            return when (dayOfWeek) {
                Calendar.MONDAY -> "星期一"
                Calendar.TUESDAY -> "星期二"
                Calendar.WEDNESDAY -> "星期三"
                Calendar.THURSDAY -> "星期四"
                Calendar.FRIDAY -> "星期五"
                Calendar.SATURDAY -> "星期六"
                else -> "星期日"
            }
        }

        private fun toMinutes(time: String): Int? {
            if (time.isBlank()) return null
            val parts = time.trim().split(":")
            if (parts.size != 2) return null
            val hour = parts[0].toIntOrNull() ?: return null
            val minute = parts[1].toIntOrNull() ?: return null
            if (hour !in 0..23 || minute !in 0..59) return null
            return hour * 60 + minute
        }

        private fun fallbackTriggerTime(): Long {
            return System.currentTimeMillis() + FALLBACK_INTERVAL_MS
        }

        private fun calculateNextTriggerTime(calendar: Calendar, nextCourseEndMinutes: Int?): Long {
            val nowMillis = System.currentTimeMillis()

            val midnight = calendar.clone() as Calendar
            midnight.add(Calendar.DAY_OF_YEAR, 1)
            midnight.set(Calendar.HOUR_OF_DAY, 0)
            midnight.set(Calendar.MINUTE, 0)
            midnight.set(Calendar.SECOND, 0)
            midnight.set(Calendar.MILLISECOND, 0)

            var candidate = minOf(midnight.timeInMillis, fallbackTriggerTime())

            if (nextCourseEndMinutes != null) {
                val endMoment = calendar.clone() as Calendar
                endMoment.set(Calendar.HOUR_OF_DAY, nextCourseEndMinutes / 60)
                endMoment.set(Calendar.MINUTE, nextCourseEndMinutes % 60)
                endMoment.set(Calendar.SECOND, 5)
                endMoment.set(Calendar.MILLISECOND, 0)
                val endMillis = endMoment.timeInMillis
                if (endMillis > nowMillis) {
                    candidate = minOf(candidate, endMillis)
                }
            }

            if (candidate <= nowMillis) {
                return nowMillis + 60_000L
            }
            return candidate
        }

        /**
         * 显示默认状态
         */
        private fun showDefaultState(views: RemoteViews) {
            views.setTextViewText(R.id.widget_header, "加载中...")
            views.setViewVisibility(R.id.courses_container, android.view.View.GONE)
            views.setViewVisibility(R.id.empty_message, android.view.View.VISIBLE)
            views.setTextViewText(R.id.empty_message, "课程结束啦 🎉")
        }

        /**
         * 安排定时刷新
         */
        fun scheduleUpdate(context: Context, triggerAtMillis: Long = fallbackTriggerTime()) {
            try {
                val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
                val intent = Intent(context, CourseWidgetProvider::class.java).apply {
                    action = ACTION_UPDATE
                }
                val pendingIntent = PendingIntent.getBroadcast(
                    context,
                    0,
                    intent,
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
                )

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                    alarmManager.setExactAndAllowWhileIdle(
                        AlarmManager.RTC_WAKEUP,
                        triggerAtMillis,
                        pendingIntent,
                    )
                } else {
                    alarmManager.setExact(
                        AlarmManager.RTC_WAKEUP,
                        triggerAtMillis,
                        pendingIntent,
                    )
                }
            } catch (e: Exception) {
                Log.e(TAG, "scheduleUpdate failed", e)
            }
        }

        private fun refreshAllWidgets(context: Context) {
            val appWidgetManager = AppWidgetManager.getInstance(context)
            val componentName = ComponentName(context, CourseWidgetProvider::class.java)
            val appWidgetIds = appWidgetManager.getAppWidgetIds(componentName)
            for (appWidgetId in appWidgetIds) {
                updateWidget(context, appWidgetManager, appWidgetId)
            }
        }
    }

    override fun onUpdate(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray,
    ) {
        for (appWidgetId in appWidgetIds) {
            updateWidget(context, appWidgetManager, appWidgetId)
        }
        scheduleUpdate(context)
    }

    override fun onReceive(context: Context, intent: Intent) {
        super.onReceive(context, intent)

        when (intent.action) {
            ACTION_UPDATE,
            Intent.ACTION_DATE_CHANGED,
            Intent.ACTION_TIME_CHANGED,
            Intent.ACTION_TIMEZONE_CHANGED,
            Intent.ACTION_BOOT_COMPLETED,
            AppWidgetManager.ACTION_APPWIDGET_UPDATE -> {
                refreshAllWidgets(context)
            }
        }
    }

    override fun onEnabled(context: Context) {
        scheduleUpdate(context)
    }

    override fun onDisabled(context: Context) {
        try {
            val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
            val intent = Intent(context, CourseWidgetProvider::class.java).apply {
                action = ACTION_UPDATE
            }
            val pendingIntent = PendingIntent.getBroadcast(
                context,
                0,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
            )
            alarmManager.cancel(pendingIntent)
        } catch (e: Exception) {
            Log.e(TAG, "onDisabled failed", e)
        }
    }
}
