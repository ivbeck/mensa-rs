plugins {
    id("com.android.application")
}

android {
    namespace = "de.ivbeck.mensa"
    compileSdk = 36

    defaultConfig {
        applicationId = "de.ivbeck.mensa"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }
}

val rustJniLibsDir = layout.projectDirectory.dir("src/main/jniLibs").asFile

tasks.register<Exec>("buildRustAndroid") {
    description = "Builds the Rust JNI library for Android ABIs."
    group = "build"
    workingDir = rootProject.projectDir.parentFile

    doFirst {
        rustJniLibsDir.mkdirs()
        val release = gradle.startParameter.taskNames.any {
            it.contains("Release", ignoreCase = true)
        }
        val args = mutableListOf(
            "ndk",
            "-t",
            "arm64-v8a",
            "-t",
            "armeabi-v7a",
            "-t",
            "x86_64",
            "-o",
            rustJniLibsDir.absolutePath,
            "build",
            "--lib",
        )
        if (release) {
            args.add("--release")
        }
        commandLine("cargo", *args.toTypedArray())
    }
}

tasks.named("preBuild") {
    dependsOn("buildRustAndroid")
}
