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
        versionCode = 1
        versionName = "0.1.0"

        ndk {
            abiFilters += listOf("arm64-v8a")
        }

        externalNativeBuild {
            cmake {
                arguments += listOf(
                    "-DANDROID_STL=none",
                    // The machine wide ~/.cargo/config.toml forces -static,
                    // which is right for a binary and wrong for a cdylib.
                    // RUSTFLAGS replaces every config file, so this has to be a
                    // real value: an empty one is *set*, and cargo would then
                    // ignore the project's own .cargo/config.toml.
                    "-DASM_CALC_RUSTFLAGS=-C link-arg=-fuse-ld=lld",
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
