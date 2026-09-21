use crate::utils::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::chipsets::mediatek::*;

pub fn performance_profile() {

    // Check if tweaks are disabled
    if is_tweak_disabled() {
        return;
    }
    
    let mut performance_gov = getprop("persist.sys.azenith.custom_performance_cpu_gov");
    if performance_gov.is_empty() {
        let avail_govs = fs::read_to_string("/sys/devices/system/cpu/cpufreq/policy0/scaling_available_governors")
            .or_else(|_| fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors"))
            .unwrap_or_default();
        if avail_govs.contains("performance") {
            performance_gov = "performance".to_string();
        } else if avail_govs.contains("schedhorizon") {
            performance_gov = "schedhorizon".to_string();
        } else if avail_govs.contains("schedplus") {
            performance_gov = "schedplus".to_string();
        } else {
            performance_gov = "schedutil".to_string();
        }
    }
    
    let lite_mode = get_litemode();

    // I/O Scheduler Tweaks
    let mut custom_perf_io = getprop("persist.sys.azenith.custom_performance_IO");
    if custom_perf_io.is_empty() {
        let mut default_io = getprop("persist.sys.azenith.custom_default_balanced_IO");
        if default_io.is_empty() {
            default_io = getprop("persist.sys.azenith.default_balanced_IO");
        }
        if default_io.is_empty() {
            default_io = "none".to_string();
        }
        custom_perf_io = default_io;
    }

    // Mali GPU Governor Tweaks
    let mut custom_perf_mali = getprop("persist.sys.azenith.custom_performance_maligpu_gov");
    if custom_perf_mali.is_empty() {
        let mut default_mali = getprop("persist.sys.azenith.custom_default_maligpu_gov");
        if default_mali.is_empty() {
            default_mali = getprop("persist.sys.azenith.default_maligpu_gov");
        }
        if default_mali.is_empty() {
            default_mali = "performance".to_string();
        }
        custom_perf_mali = default_mali;
    }

    apply_custom_governor_io(&performance_gov, &custom_perf_io, &custom_perf_mali);

    if Path::new("/proc/ppm").exists() {
        setgamefreqppm();
    } else {
        setgamefreq();
    }

    if !lite_mode {
        log_info("Set CPU freq to max available Frequencies");
    } else {
        log_info("Set CPU freq to normal Frequencies");
    }

    // VM Tweaks for Stutter-Free Gaming
    write_lock("70", "/proc/sys/vm/vfs_cache_pressure");
    write_lock("60", "/proc/sys/vm/swappiness");
    write_lock("10", "/proc/sys/vm/stat_interval");
    write_lock("0", "/proc/sys/vm/compaction_proactiveness");
    write_lock("0", "/proc/sys/vm/page-cluster");
    write_lock("20", "/proc/sys/vm/dirty_ratio");
    write_lock("5", "/proc/sys/vm/dirty_background_ratio");
    write_lock("500", "/proc/sys/vm/dirty_expire_centisecs");
    write_lock("100", "/proc/sys/vm/dirty_writeback_centisecs");

    write_lock("N", "/sys/module/workqueue/parameters/power_efficient");
    write_lock("0", "/sys/devices/system/cpu/eas/enable");

    // Schedtune boost for instant frame delivery
    if let Ok(paths) = glob::glob("/dev/stune/*") {
        for path in paths.flatten() {
            if path.is_dir() {
                let p_str = path.to_str().unwrap();
                let is_top_app = p_str.ends_with("top-app");
                let boost_val = if is_top_app { "50" } else { "30" };
                let prefer_idle_val = if is_top_app { "1" } else { "0" };

                write_lock(boost_val, &format!("{}/schedtune.boost", p_str));
                write_lock("1", &format!("{}/schedtune.sched_boost_enabled", p_str));
                write_lock(prefer_idle_val, &format!("{}/schedtune.prefer_idle", p_str));
                write_lock("0", &format!("{}/schedtune.colocate", p_str));
            }
        }
    }

    // Ensure top-app has all cores available
    if let Ok(cores) = fs::read_to_string("/sys/devices/system/cpu/possible") {
        let cores_trimmed = cores.trim();
        if !cores_trimmed.is_empty() {
            write_lock(cores_trimmed, "/dev/cpuset/top-app/cpus");
            write_lock(cores_trimmed, "/dev/cpuset/foreground/cpus");
        }
    }

    // Network Tweaks for Ultra-Low Ping & Anti-Jitter in Online Games
    write_lock("1", "/proc/sys/net/ipv4/tcp_low_latency");
    write_lock("0", "/proc/sys/net/ipv4/tcp_slow_start_after_idle");
    write_lock("3", "/proc/sys/net/ipv4/tcp_fastopen");
    write_lock("1", "/proc/sys/net/ipv4/tcp_tw_reuse");
    write_lock("1", "/proc/sys/net/ipv4/tcp_sack");
    write_lock("1", "/proc/sys/net/ipv4/tcp_dsack");
    write_lock("8388608", "/proc/sys/net/core/rmem_max");
    write_lock("8388608", "/proc/sys/net/core/wmem_max");
    write_lock("4194304", "/proc/sys/net/core/rmem_default");
    write_lock("4194304", "/proc/sys/net/core/wmem_default");
    write_lock("5000", "/proc/sys/net/core/netdev_max_backlog");

    // Framework Touch Responsiveness & Latency Reduction
    setprop("touch.pressure.scale", "0.001");
    setprop("touch.size.calibration", "geometric");
    setprop("view.touch_slop", "2");
    setprop("view.scroll_friction", "0.004");
    setprop("ro.input.resampling", "1");
    setprop("debug.input.resampling", "1");
    setprop("debug.input.resampling.use_frames", "1");
    setprop("debug.touch.sampling", "1");
    setprop("persist.sys.touch.smooth", "1");

    // Re-enforce Game Engine Libs Priority
    let libs = "libunity.so, libil2cpp.so, libmain.so, libUE4.so, libUE5.so, libvulkan.so, libGLESv2.so, libanort.so, libgodot_android.so, libgdx.so, libgdx-box2d.so, libminecraftpe.so, libLive2DCubismCore.so, libyuzu-android.so, libryujinx.so, libcitra-android.so, libhdr_pro_engine.so, libandroidx.graphics.path.so, libeffect.so";
    write_lock(libs, "/proc/sys/kernel/sched_lib_name");
    write_lock("255", "/proc/sys/kernel/sched_lib_mask_force");

    let bs_path = "/sys/module/battery_saver/parameters/enabled";
    if Path::new(bs_path).exists() {
        let content = fs::read_to_string(bs_path).unwrap_or_default();
        if content.chars().any(|c: char| c.is_ascii_digit()) {
            write_lock("0", bs_path);
        } else {
            write_lock("N", bs_path);
        }
    }

    write_lock("0", "/proc/sys/kernel/split_lock_mitigate");

    let sched_feat = "/sys/kernel/debug/sched_features";
    if Path::new(sched_feat).exists() {
        write_lock("NEXT_BUDDY", sched_feat);
        write_lock("NO_TTWU_QUEUE", sched_feat);
    }
    
    // Fast I/O Storage Tweaks for Game Asset Streaming
    std::thread::spawn(|| {
        if let Ok(paths) = glob::glob("/sys/block/*") {
            for path in paths.flatten() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("mmcblk") || file_name.starts_with("sd") || file_name.starts_with("dm-") {
                        if let Some(p_str) = path.to_str() {
                            write_lock("1024", &format!("{}/queue/read_ahead_kb", p_str));
                            write_lock("256", &format!("{}/queue/nr_requests", p_str));
                            write_lock("0", &format!("{}/queue/iostats", p_str));
                            write_lock("2", &format!("{}/queue/rq_affinity", p_str));
                            write_lock("0", &format!("{}/queue/add_random", p_str));
                            write_lock("0", &format!("{}/queue/rotational", p_str));
                        }
                    }
                }
            }
        }
    });

    if get_clearapps() {
        clear_background_apps();
    }

    if !lite_mode {
        mediatek_performance();
    }

    log_verbose("Performance Profile Applied Successfully!");
}

pub fn balanced_profile() {

    // Check if tweaks are disabled
    if is_tweak_disabled() {
        return;
    }
    
    let mut default_gov = getprop("persist.sys.azenith.custom_default_cpu_gov");
    if default_gov.is_empty() {
        default_gov = getprop("persist.sys.azenith.default_cpu_gov");
    }
    if default_gov.is_empty() {
        default_gov = "schedutil".to_string();
    }

    // I/O Scheduler Tweaks
    let mut default_io = getprop("persist.sys.azenith.custom_default_balanced_IO");
    if default_io.is_empty() {
        default_io = getprop("persist.sys.azenith.default_balanced_IO");
    }
    if default_io.is_empty() {
        default_io = "none".to_string();
    }

    // Mali GPU Governor Tweaks
    let mut default_mali = getprop("persist.sys.azenith.custom_default_maligpu_gov");
    if default_mali.is_empty() {
        default_mali = getprop("persist.sys.azenith.default_maligpu_gov");
    }

    apply_custom_governor_io(&default_gov, &default_io, &default_mali);

    if Path::new("/proc/ppm").exists() {
        setfreqppm();
    } else {
        setfreq();
    }

    if getprop("persist.sys.azenithconf.freqoffset") == "Disabled" {
        log_info("Set CPU freq to normal Frequencies");
    } else {
        log_info("Set CPU freq to normal selected Frequencies");
    }

    write_lock("120", "/proc/sys/vm/vfs_cache_pressure");
    write_lock("Y", "/sys/module/workqueue/parameters/power_efficient");
    write_lock("1", "/sys/devices/system/cpu/eas/enable");

    if let Ok(paths) = glob::glob("/dev/stune/*") {
        for path in paths.flatten() {
            if path.is_dir() {
                let p_str = path.to_str().unwrap();
                write_lock("0", &format!("{}/schedtune.boost", p_str));
                write_lock("0", &format!("{}/schedtune.sched_boost_enabled", p_str));
                write_lock("0", &format!("{}/schedtune.prefer_idle", p_str));
                write_lock("0", &format!("{}/schedtune.colocate", p_str));
            }
        }
    }

    let bs_path = "/sys/module/battery_saver/parameters/enabled";
    if Path::new(bs_path).exists() {
        let content = fs::read_to_string(bs_path).unwrap_or_default();
        if content.chars().any(|c: char| c.is_ascii_digit()) {
            write_lock("0", bs_path);
        } else {
            write_lock("N", bs_path);
        }
    }

    write_lock("1", "/proc/sys/kernel/split_lock_mitigate");

    let sched_feat = "/sys/kernel/debug/sched_features";
    if Path::new(sched_feat).exists() {
        write_lock("NEXT_BUDDY", sched_feat);
        write_lock("TTWU_QUEUE", sched_feat);
    }
    
    // I/O Tweaks
    std::thread::spawn(|| {
        if let Ok(paths) = glob::glob("/sys/block/*") {
            for path in paths.flatten() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name == "mmcblk0" || file_name == "mmcblk1" || file_name.starts_with("sd") {
                        if let Some(p_str) = path.to_str() {
                            write_lock("128", &format!("{}/queue/read_ahead_kb", p_str));
                            write_lock("64", &format!("{}/queue/nr_requests", p_str));
                        }
                    }
                }
            }
        }
    });

    mediatek_balance();

    log_verbose("Balanced Profile applied successfully!");
}

pub fn eco_mode() {

    // Check if tweaks are disabled
    if is_tweak_disabled() {
        return;
    }
    
    let mut powersave_gov = getprop("persist.sys.azenith.custom_powersave_cpu_gov");
    if powersave_gov.is_empty() {
        powersave_gov = "powersave".to_string();
    }

    // I/O Scheduler Tweaks
    let mut powersave_io = getprop("persist.sys.azenith.custom_powersave_IO");
    if powersave_io.is_empty() {
        powersave_io = "none".to_string();
    }

    // Mali GPU Governor Tweaks
    let mut custom_eco_mali = getprop("persist.sys.azenith.custom_powersave_maligpu_gov");
    if custom_eco_mali.is_empty() {
        let mut default_mali = getprop("persist.sys.azenith.custom_default_maligpu_gov");
        if default_mali.is_empty() {
            default_mali = getprop("persist.sys.azenith.default_maligpu_gov");
        }
        custom_eco_mali = default_mali;
    }

    apply_custom_governor_io(&powersave_gov, &powersave_io, &custom_eco_mali);

    if Path::new("/proc/ppm").exists() {
        setfreqppm();
    } else {
        setfreq();
    }
    log_info("Set CPU freq to low Frequencies");

    write_lock("120", "/proc/sys/vm/vfs_cache_pressure");
    write_lock("Y", "/sys/module/workqueue/parameters/power_efficient");
    write_lock("1", "/sys/devices/system/cpu/eas/enable");

    if let Ok(paths) = glob::glob("/dev/stune/*") {
        for path in paths.flatten() {
            if path.is_dir() {
                let p_str = path.to_str().unwrap();
                write_lock("0", &format!("{}/schedtune.boost", p_str));
                write_lock("0", &format!("{}/schedtune.sched_boost_enabled", p_str));
                write_lock("0", &format!("{}/schedtune.prefer_idle", p_str));
                write_lock("0", &format!("{}/schedtune.colocate", p_str));
            }
        }
    }

    let bs_path = "/sys/module/battery_saver/parameters/enabled";
    if Path::new(bs_path).exists() {
        let content = fs::read_to_string(bs_path).unwrap_or_default();
        if content.chars().any(|c| c.is_ascii_digit()) {
            write_lock("1", bs_path);
        } else {
            write_lock("Y", bs_path);
        }
    }

    write_lock("1", "/proc/sys/kernel/split_lock_mitigate");

    let sched_feat = "/sys/kernel/debug/sched_features";
    if Path::new(sched_feat).exists() {
        write_lock("NO_NEXT_BUDDY", sched_feat);
        write_lock("NO_TTWU_QUEUE", sched_feat);
    }

    mediatek_powersave();

    log_verbose("ECO Mode applied successfully!");
}

pub fn initialize() {
    // Initial kernel panics & sync
    for param in &["panic", "panic_on_warn", "panic_on_oops", "softlockup_panic"] {
        write_lock("0", &format!("/proc/sys/kernel/{}", param));
    }
    let _ = Command::new("sync").status();
    
    // Display / SurfaceFlinger config
    let scheme = getprop("persist.sys.azenithconf.schemeconfig");
    if scheme != "1000 1000 1000 1000" && !scheme.is_empty() {
        let parts: Vec<&str> = scheme.split_whitespace().collect();
        if parts.len() >= 4 {
            let r = parts[0].parse::<f32>().unwrap_or(1000.0) / 1000.0;
            let g = parts[1].parse::<f32>().unwrap_or(1000.0) / 1000.0;
            let b = parts[2].parse::<f32>().unwrap_or(1000.0) / 1000.0;
            let s = parts[3].parse::<f32>().unwrap_or(1000.0) / 1000.0;

            let _ = Command::new("service").args([
                "call", "SurfaceFlinger", "1015", "i32", "1",
                "f", &r.to_string(), "f", "0", "f", "0", "f", "0",
                "f", "0", "f", &g.to_string(), "f", "0", "f", "0",
                "f", "0", "f", "0", "f", &b.to_string(), "f", "0",
                "f", "0", "f", "0", "f", "0", "f", "1"
            ]).status();

            let _ = Command::new("service").args([
                "call", "SurfaceFlinger", "1022", "f", &s.to_string()
            ]).status();
        }
    }
    
    // Check if tweaks are disabled
    if is_tweak_disabled() {
        return;
    }

    // Initialize CPU & I/O & Mali GPU
    init_cpu_governor();
    init_io_scheduler();
    init_maligpu_governor();
    init_renderer();
    
    // Thermal governor
    if let Ok(paths) = glob::glob("/sys/class/thermal/thermal_zone*") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("step_wise", &format!("{}/policy", p_str));
            }
        }
    }
    
    // I/O Tweaks
    if let Ok(paths) = glob::glob("/sys/block/*") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("0", &format!("{}/queue/iostats", p_str));
                write_lock("0", &format!("{}/queue/add_random", p_str));
            }
        }
    }

    // Networking tweaks
    let tcp_avail = fs::read_to_string("/proc/sys/net/ipv4/tcp_available_congestion_control").unwrap_or_default();
    let algos = ["bbr3", "bbr2", "bbrplus", "bbr", "westwood", "cubic"];
    for algo in algos.iter() {
        if tcp_avail.contains(algo) {
            write_lock(algo, "/proc/sys/net/ipv4/tcp_congestion_control");
            break;
        }
    }

    write_lock("1", "/proc/sys/net/ipv4/tcp_low_latency");
    write_lock("1", "/proc/sys/net/ipv4/tcp_ecn");
    write_lock("3", "/proc/sys/net/ipv4/tcp_fastopen");
    write_lock("1", "/proc/sys/net/ipv4/tcp_sack");
    write_lock("0", "/proc/sys/net/ipv4/tcp_timestamps");

    // General Kernel & Scheduler Tweaks
    write_lock("3", "/proc/sys/kernel/perf_cpu_time_max_percent");
    write_lock("0", "/proc/sys/kernel/sched_schedstats");
    write_lock("0", "/proc/sys/kernel/task_cpustats_enable");
    write_lock("0", "/proc/sys/kernel/sched_autogroup_enabled");
    write_lock("1", "/proc/sys/kernel/sched_child_runs_first");
    write_lock("32", "/proc/sys/kernel/sched_nr_migrate");
    write_lock("50000", "/proc/sys/kernel/sched_migration_cost_ns");
    write_lock("1000000", "/proc/sys/kernel/sched_min_granularity_ns");
    write_lock("1500000", "/proc/sys/kernel/sched_wakeup_granularity_ns");

    // VM Tweaks
    write_lock("0", "/proc/sys/vm/page-cluster");
    write_lock("15", "/proc/sys/vm/stat_interval");
    write_lock("0", "/proc/sys/vm/compaction_proactiveness");

    // Vendor Bloats & Module Tweaks
    write_lock("0", "/sys/module/mmc_core/parameters/use_spi_crc");
    write_lock("0", "/sys/module/opchain/parameters/chain_on");
    write_lock("0", "/sys/module/cpufreq_bouncing/parameters/enable");
    write_lock("0", "/proc/task_info/task_sched_info/task_sched_info_enable");
    write_lock("0", "/proc/oplus_scheduler/sched_assist/sched_assist_enabled");

    // Libraries Max Perf Reporting
    let libs = "libunity.so, libil2cpp.so, libmain.so, libUE4.so, libUE5.so, libvulkan.so, libGLESv2.so, libanort.so, libgodot_android.so, libgdx.so, libgdx-box2d.so, libminecraftpe.so, libLive2DCubismCore.so, libyuzu-android.so, libryujinx.so, libcitra-android.so, libhdr_pro_engine.so, libandroidx.graphics.path.so, libeffect.so";
    write_lock(libs, "/proc/sys/kernel/sched_lib_name");
    write_lock("255", "/proc/sys/kernel/sched_lib_mask_force");

    systemv("sys.azenith-utilityconf FSTrim");
    systemv("sh /data/adb/modules/AZenith/preferenced-tweaks.sh");
    
    // Final Sync & Logs
    let _ = Command::new("sync").status();
    log_verbose("Initializing Complete");
    log_info("Initializing Complete");
}
