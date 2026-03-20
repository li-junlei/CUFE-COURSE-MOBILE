package com.lijunlei.cufecourse

import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  companion object {
    private const val TAG = "MainActivity"
    private const val WIDGET_UPDATE_THROTTLE_MS = 30000L // 30 seconds
  }

  private var lastWidgetUpdateTime = 0L

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  override fun onResume() {
    super.onResume()
    triggerWidgetUpdate()
  }

  private fun triggerWidgetUpdate() {
    val now = System.currentTimeMillis()
    if (now - lastWidgetUpdateTime < WIDGET_UPDATE_THROTTLE_MS) {
      return
    }

    lastWidgetUpdateTime = now
    try {
      val intent = Intent(this, CourseWidgetProvider::class.java).apply {
        action = CourseWidgetProvider.ACTION_UPDATE
      }
      sendBroadcast(intent)
    } catch (e: Exception) {
      Log.w(TAG, "Failed to trigger widget update", e)
    }
  }
}
