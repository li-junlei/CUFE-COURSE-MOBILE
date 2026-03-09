import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

// 签名配置 - 优先使用环境变量/gradle属性，否则使用默认值
val releaseKeyAlias: String = findProperty("cufe_course_mobile.release.keyAlias") as? String 
    ?: System.getenv("ORG_GRADLE_PROJECT_cufe_course_mobile_RELEASE_KEY_ALIAS") 
    ?: "cufe-course"
val releaseKeyPassword: String = findProperty("cufe_course_mobile.release.keyPassword") as? String 
    ?: System.getenv("ORG_GRADLE_PROJECT_cufe_course_mobile_RELEASE_KEY_PASSWORD") 
    ?: "cufe2024"
val releaseStoreFileName: String = findProperty("cufe_course_mobile.release.storeFile") as? String 
    ?: System.getenv("ORG_GRADLE_PROJECT_cufe_course_mobile_RELEASE_STORE_FILE") 
    ?: "release.keystore"
val releaseStorePassword: String = findProperty("cufe_course_mobile.release.storePassword") as? String 
    ?: System.getenv("ORG_GRADLE_PROJECT_cufe_course_mobile_RELEASE_STORE_PASSWORD") 
    ?: "cufe2024"

android {
    // 在android块级别创建signingConfigs
    signingConfigs {
        create("release") {
            keyAlias = releaseKeyAlias
            keyPassword = releaseKeyPassword
            storeFile = file(releaseStoreFileName)
            storePassword = releaseStorePassword
        }
    }
    compileSdk = 36
    namespace = "com.lijunlei.cufecourse"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "com.lijunlei.cufecourse"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
        ndk {
            abiFilters.add("arm64-v8a")
        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")

