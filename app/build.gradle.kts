import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.application)
}

android {
    namespace = "dev.nga.asmcalc"
    compileSdk = 36
    ndkVersion = "27.1.12297006"

    defaultConfig {
        applicationId = "dev.nga.asmcalc"
        minSdk = 21
        targetSdk = 36
        versionCode = 2
        versionName = "0.2.0"

        ndk {
            abiFilters += listOf("arm64-v8a")
        }

        externalNativeBuild {
            cmake {
                arguments += listOf(
                    "-DANDROID_STL=none",
                    // RUSTFLAGS replaces every cargo config file, which is how
                    // the machine wide ~/.cargo/config.toml is overridden: it
                    // forces -static, right for a static executable and wrong
                    // for a cdylib.
                    //
                    // A *real* value is required. An empty RUSTFLAGS is still
                    // "set", and cargo would then ignore this project's own
                    // .cargo/config.toml, losing the linker script.
                    "-DASM_CALC_RUSTFLAGS=-C debuginfo=0",
                )
            }
        }
    }

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
            version = "3.22.1"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}
