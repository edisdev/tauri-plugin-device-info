package com.tauri.plugin.deviceinfo

import android.app.Activity
import android.os.Build
import android.provider.Settings
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@TauriPlugin
class DeviceInfo(private val activity: Activity) : Plugin(activity) {
  companion object {
    private const val TAG = "DeviceInfoPlugin" // Log tag'i tanımlıyoruz
  }

  @Command
  fun get_device_info(invoke: Invoke) {
    try {
      val androidId =
              Settings.Secure.getString(activity.contentResolver, Settings.Secure.ANDROID_ID)
      val deviceName =
              Settings.Global.getString(activity.contentResolver, Settings.Global.DEVICE_NAME)
                      ?: Settings.Secure.getString(activity.contentResolver, "bluetooth_name")

      val info =
              mapOf(
                      "uuid" to androidId,
                      "manufacturer" to Build.MANUFACTURER,
                      "model" to Build.MODEL,
                      "serial" to androidId,
                      "android_id" to androidId,
                      "device_name" to deviceName
              )

      Log.d(TAG, "Device Info: $info")

      val jsObject = JSObject()
      info.forEach { (key, value) -> jsObject.put(key, value) }

      Log.d(TAG, "Device Info jsObject: $jsObject")

      invoke.resolve(jsObject)
    } catch (e: Exception) {
      invoke.reject("Failed to get device info: ${e.message}")
    }
  }

  @Command
  fun get_battery_info(invoke: Invoke) {
    try {
      val batteryManager =
              activity.getSystemService(android.content.Context.BATTERY_SERVICE) as
                      android.os.BatteryManager
      val level =
              batteryManager
                      .getIntProperty(android.os.BatteryManager.BATTERY_PROPERTY_CAPACITY)
                      .toFloat()

      // Use Intent-based approach for reliable charging check on all Android versions
      val ifilter = android.content.IntentFilter(android.content.Intent.ACTION_BATTERY_CHANGED)
      val batteryStatus = activity.registerReceiver(null, ifilter)

      val status = batteryStatus?.getIntExtra(android.os.BatteryManager.EXTRA_STATUS, -1) ?: -1
      val isChargingBool =
              status == android.os.BatteryManager.BATTERY_STATUS_CHARGING ||
                      status == android.os.BatteryManager.BATTERY_STATUS_FULL

      val healthInt =
              batteryStatus?.getIntExtra(
                      android.os.BatteryManager.EXTRA_HEALTH,
                      android.os.BatteryManager.BATTERY_HEALTH_UNKNOWN
              )
                      ?: -1

      val healthString =
              when (healthInt) {
                android.os.BatteryManager.BATTERY_HEALTH_GOOD -> "Good"
                android.os.BatteryManager.BATTERY_HEALTH_OVERHEAT -> "Overheat"
                android.os.BatteryManager.BATTERY_HEALTH_DEAD -> "Dead"
                android.os.BatteryManager.BATTERY_HEALTH_OVER_VOLTAGE -> "Over Voltage"
                android.os.BatteryManager.BATTERY_HEALTH_UNSPECIFIED_FAILURE ->
                        "Unspecified Failure"
                android.os.BatteryManager.BATTERY_HEALTH_COLD -> "Cold"
                else -> "Unknown"
              }

      val jsObject = JSObject()
      jsObject.put("level", level)
      jsObject.put("isCharging", isChargingBool)
      jsObject.put("health", healthString)

      invoke.resolve(jsObject)
    } catch (e: Exception) {
      invoke.reject("Failed to get battery info: ${e.message}")
    }
  }

  @Command
  fun get_network_info(invoke: Invoke) {
    try {
      val connectivityManager =
              activity.getSystemService(android.content.Context.CONNECTIVITY_SERVICE) as
                      android.net.ConnectivityManager
      val activeNetwork = connectivityManager.activeNetwork
      val capabilities = connectivityManager.getNetworkCapabilities(activeNetwork)

      val networkType =
              when {
                capabilities?.hasTransport(android.net.NetworkCapabilities.TRANSPORT_WIFI) ==
                        true -> "wifi"
                capabilities?.hasTransport(android.net.NetworkCapabilities.TRANSPORT_CELLULAR) ==
                        true -> "cellular"
                capabilities?.hasTransport(android.net.NetworkCapabilities.TRANSPORT_ETHERNET) ==
                        true -> "ethernet"
                else -> "unknown"
              }

      val linkProperties = connectivityManager.getLinkProperties(activeNetwork)
      val ipAddress =
              linkProperties?.linkAddresses
                      ?.firstOrNull { it.address is java.net.Inet4Address }
                      ?.address
                      ?.hostAddress
                      ?: "0.0.0.0"

      val jsObject = JSObject()
      jsObject.put("networkType", networkType)
      jsObject.put("ipAddress", ipAddress)
      jsObject.put("macAddress", "restricted")

      invoke.resolve(jsObject)
    } catch (e: Exception) {
      invoke.reject("Failed to get network info: ${e.message}")
    }
  }

  @Command
  fun get_storage_info(invoke: Invoke) {
    try {
      val stat = android.os.StatFs(android.os.Environment.getDataDirectory().path)
      val total = stat.totalBytes
      val free = stat.availableBytes

      val jsObject = JSObject()
      jsObject.put("totalSpace", total)
      jsObject.put("freeSpace", free)
      jsObject.put("storageType", "internal")

      invoke.resolve(jsObject)
    } catch (e: Exception) {
      invoke.reject("Failed to get storage info: ${e.message}")
    }
  }

  @Command
  fun get_display_info(invoke: Invoke) {
    try {
      val metrics = activity.resources.displayMetrics

      val jsObject = JSObject()
      jsObject.put("width", metrics.widthPixels)
      jsObject.put("height", metrics.heightPixels)
      jsObject.put("scaleFactor", metrics.density)
      jsObject.put(
              "refreshRate",
              if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
                activity.display?.refreshRate ?: 60.0f
              } else {
                activity.windowManager.defaultDisplay.refreshRate
              }
      )

      invoke.resolve(jsObject)
    } catch (e: Exception) {
      invoke.reject("Failed to get display info: ${e.message}")
    }
  }
  @Command
  fun get_power_state(invoke: Invoke) {
    try {
      val powerManager =
              activity.getSystemService(android.content.Context.POWER_SERVICE) as
                      android.os.PowerManager

      val jsObject = JSObject()
      jsObject.put("lowPowerMode", powerManager.isPowerSaveMode)

      // Thermal status exists from Android 10 (API 29); map Android's 7 levels
      // onto the plugin's 4-level (Apple) scale.
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        val thermal =
                when (powerManager.currentThermalStatus) {
                  android.os.PowerManager.THERMAL_STATUS_NONE -> "nominal"
                  android.os.PowerManager.THERMAL_STATUS_LIGHT,
                  android.os.PowerManager.THERMAL_STATUS_MODERATE -> "fair"
                  android.os.PowerManager.THERMAL_STATUS_SEVERE -> "serious"
                  android.os.PowerManager.THERMAL_STATUS_CRITICAL,
                  android.os.PowerManager.THERMAL_STATUS_EMERGENCY,
                  android.os.PowerManager.THERMAL_STATUS_SHUTDOWN -> "critical"
                  else -> null
                }
        if (thermal != null) jsObject.put("thermalState", thermal)
      }

      invoke.resolve(jsObject)
    } catch (e: Exception) {
      invoke.reject("Failed to get power state: ${e.message}")
    }
  }

  @Command
  fun get_hardware_info(invoke: Invoke) {
    try {
      val activityManager =
              activity.getSystemService(android.content.Context.ACTIVITY_SERVICE) as
                      android.app.ActivityManager
      val memoryInfo = android.app.ActivityManager.MemoryInfo()
      activityManager.getMemoryInfo(memoryInfo)

      val jsObject = JSObject()
      jsObject.put("cpuCores", Runtime.getRuntime().availableProcessors())
      jsObject.put("totalMemory", memoryInfo.totalMem)
      jsObject.put("isMobile", true)
      jsObject.put("isLowRamDevice", activityManager.isLowRamDevice)

      val maxMhz = readCpuMaxFrequencyMhz()
      if (maxMhz != null) jsObject.put("cpuMaxFrequencyMhz", maxMhz)

      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        jsObject.put("cpuModel", Build.SOC_MODEL.takeUnless { it == Build.UNKNOWN } ?: Build.HARDWARE)
        // 0 means the device declares no media performance class.
        val mpc = Build.VERSION.MEDIA_PERFORMANCE_CLASS
        if (mpc > 0) jsObject.put("mediaPerformanceClass", mpc)
      } else {
        jsObject.put("cpuModel", Build.HARDWARE)
      }

      invoke.resolve(jsObject)
    } catch (e: Exception) {
      invoke.reject("Failed to get hardware info: ${e.message}")
    }
  }

  /** Highest `cpuinfo_max_freq` across cores in MHz (big.LITTLE: the prime core), or null. */
  private fun readCpuMaxFrequencyMhz(): Long? {
    val cpus = java.io.File("/sys/devices/system/cpu").listFiles { f -> f.name.matches(Regex("cpu\\d+")) }
            ?: return null
    return cpus.mapNotNull { cpu ->
              try {
                java.io.File(cpu, "cpufreq/cpuinfo_max_freq").readText().trim().toLong() / 1000
              } catch (e: Exception) {
                null
              }
            }
            .maxOrNull()
  }
}
