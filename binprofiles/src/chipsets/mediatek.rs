use crate::utils::*;
use std::fs;
use std::path::Path;

/// Applies MediaTek-specific Balanced Profile tuning
pub fn mediatek_balance() {
    // 1. PPM (Processor Power Management)
    if Path::new("/proc/ppm/policy_status").exists() {
        let content = fs::read_to_string("/proc/ppm/policy_status").unwrap_or_default();
        for line in content.lines() {
            let is_target_1 = line.contains("FORCE_LIMIT") || line.contains("PWR_THRO") || line.contains("THERMAL") || line.contains("USER_LIMIT");
            let is_target_2 = line.contains("SYS_BOOST");
            if is_target_1 || is_target_2 {
                if let Some(idx_str) = line.split('[').nth(1).and_then(|s| s.split(']').next()) {
                    if is_target_1 {
                        write_lock(&format!("{} 1", idx_str), "/proc/ppm/policy_status");
                    }
                    if is_target_2 {
                        write_lock(&format!("{} 0", idx_str), "/proc/ppm/policy_status");
                    }
                }
            }
        }
    }
    ppm_fix_freq("-1");

    // Enable CPU hotplugging back for balanced power efficiency
    write_lock("1", "/proc/hps/enabled");

    // 2. CPUFreq & EAS
    write_lock("0", "/proc/cpufreq/cpufreq_cci_mode");
    write_lock("1", "/proc/cpufreq/cpufreq_power_mode");
    write_lock("1", "/sys/devices/system/cpu/eas/enable");

    // 3. FPSGO & GED Balanced
    write_lock("2", "/sys/kernel/fpsgo/common/force_onoff");
    write_lock("1", "/sys/module/sspm_v3/holders/ged/parameters/is_GED_KPI_enabled");
    write_lock("0", "/sys/module/ged/parameters/gx_game_mode");
    write_lock("0", "/sys/module/ged/parameters/gx_force_cpu_boost");
    write_lock("0", "/sys/module/ged/parameters/gx_boost_on");
    write_lock("50", "/sys/kernel/ged/hal/gpu_boost_level");
    write_lock("0", "/sys/pnpmgr/fpsgo_boost/boost_enable");
    write_lock("0", "/sys/pnpmgr/fpsgo_boost/boost_mode");

    // 4. Memory & DRAM Boost
    write_lock("0", "/sys/devices/platform/boot_dramboost/dramboost/dramboost");

    // 5. GPU Tuning (Mali & PowerVR)
    if Path::new("/proc/gpufreq").exists() {
        write_lock("0", "/proc/gpufreq/gpufreq_opp_freq");
    } else if Path::new("/proc/gpufreqv2").exists() {
        write_lock("-1", "/proc/gpufreqv2/fix_target_opp_index");
    }

    if Path::new("/proc/gpufreq/gpufreq_power_limited").exists() {
        let settings = [
            "ignore_batt_oc", "ignore_batt_percent", "ignore_low_batt",
            "ignore_thermal_protect", "ignore_pbm_limited"
        ];
        for setting in settings {
            write_lock(&format!("{} 0", setting), "/proc/gpufreq/gpufreq_power_limited");
        }
    }

    if let Ok(mut paths) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(path)) = paths.next() {
            write_lock("coarse_demand", &format!("{}/power_policy", path.display()));
        }
    }
    if let Ok(paths) = glob::glob("/sys/class/devfreq/*mali*/governor") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("simple_ondemand", p_str);
            }
        }
    }
    if Path::new("/sys/module/pvrsrvkm/parameters/gpu_dvfs_enable").exists() {
        write_lock("1", "/sys/module/pvrsrvkm/parameters/gpu_dvfs_enable");
    }

    // 6. DVFSRC (Dynamic Voltage and Frequency Scaling Resource Collector)
    write_lock("-1", "/sys/kernel/helio-dvfsrc/dvfsrc_force_vcore_dvfs_opp");
    write_lock("userspace", "/sys/class/devfreq/mtk-dvfsrc-devfreq/governor");

    if let Ok(paths) = glob::glob("/sys/devices/platform/*.dvfsrc") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("-1", &format!("{}/helio-dvfsrc/dvfsrc_req_ddr_opp", p_str));
            }
        }
    }

    if let Ok(paths) = glob::glob("/sys/devices/platform/soc/*.dvfsrc") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("userspace", &format!("{}/mtk-dvfsrc-devfreq/devfreq/mtk-dvfsrc-devfreq/governor", p_str));
            }
        }
    }

    // 7. Thermal & Limits
    write_lock("0", "/proc/perfmgr/syslimiter/syslimiter_force_disable");
    write_lock("stop 0", "/proc/mtk_batoc_throttling/battery_oc_protect_stop");
    write_lock("1", "/sys/kernel/eara_thermal/enable");
    write_lock("1", "/proc/thermal/clatm_enable");
}

/// Applies MediaTek-specific Maximum Performance Profile tuning for Ultra-Smooth Gaming
pub fn mediatek_performance() {
    // 1. PPM (Processor Power Management) - Complete Unthrottling
    if Path::new("/proc/ppm/policy_status").exists() {
        let content = fs::read_to_string("/proc/ppm/policy_status").unwrap_or_default();
        for line in content.lines() {
            let is_target_1 = line.contains("FORCE_LIMIT") || line.contains("PWR_THRO") || line.contains("THERMAL") || line.contains("USER_LIMIT");
            let is_target_2 = line.contains("SYS_BOOST");
            if is_target_1 || is_target_2 {
                if let Some(idx_str) = line.split('[').nth(1).and_then(|s| s.split(']').next()) {
                    if is_target_1 {
                        // Disable throttle limits (0)
                        write_lock(&format!("{} 0", idx_str), "/proc/ppm/policy_status");
                    }
                    if is_target_2 {
                        // Enable system boost (1)
                        write_lock(&format!("{} 1", idx_str), "/proc/ppm/policy_status");
                    }
                }
            }
        }
    }
    ppm_fix_freq("0");

    if Path::new("/proc/ppm/mode").exists() {
        write_lock("0", "/proc/ppm/mode");
    }
    if Path::new("/proc/ppm/root_cluster").exists() {
        write_lock("0", "/proc/ppm/root_cluster");
    }

    // Keep all CPU cores online (Disable HPS aggressive hotplug sleeping)
    write_lock("0", "/proc/hps/enabled");
    write_lock("60", "/proc/hps/up_threshold");
    write_lock("30", "/proc/hps/down_threshold");

    // 2. CPUFreq, CCI & EAS Optimization
    write_lock("1", "/proc/cpufreq/cpufreq_cci_mode"); // Boost Cache Coherent Interconnect bandwidth
    write_lock("3", "/proc/cpufreq/cpufreq_power_mode"); // Max performance power mode
    write_lock("0", "/proc/cpufreq/cpufreq_sched_disable");
    write_lock("0", "/sys/devices/system/cpu/eas/enable");

    // 3. Ultra FPSGO & GED Frame Stabilization
    let use_fpsgo = getprop("persist.sys.azenithconf.usefpsgo");
    if use_fpsgo == "0" {
        write_lock("0", "/sys/kernel/fpsgo/common/force_onoff");
    } else {
        write_lock("1", "/sys/kernel/fpsgo/common/force_onoff");
    }

    // Direct FPSGO knobs to prevent frame drops
    let fpsgo_dir = "/sys/kernel/fpsgo";
    write_lock("1", &format!("{}/common/fpsgo_enable", fpsgo_dir));
    write_lock("1", &format!("{}/fstb/fpsgo_status", fpsgo_dir));
    write_lock("1", &format!("{}/fbt/boost_ta", fpsgo_dir));           // Boost top-app
    write_lock("1", &format!("{}/fbt/boost_VIP", fpsgo_dir));          // Boost VIP render threads
    write_lock("0", &format!("{}/fbt/switch_idle", fpsgo_dir));
    write_lock("1", &format!("{}/fbt/loading_enable", fpsgo_dir));
    write_lock("0", &format!("{}/fbt/enable_switch_down_throttle", fpsgo_dir)); // Disable down-throttling!
    write_lock("0", &format!("{}/fbt/thrm_limit_cpu", fpsgo_dir));     // Do not drop CPU freq on thermals!
    write_lock("0", &format!("{}/fbt/thrm_sub_cpu", fpsgo_dir));
    write_lock("0", &format!("{}/fstb/gpu_slowdown_check", fpsgo_dir)); // Do not throttle on GPU busy!
    write_lock("1", &format!("{}/fstb/adopt_low_fps", fpsgo_dir));
    write_lock("1", &format!("{}/fstb/fstb_self_ctrl_fps_enable", fpsgo_dir));
    write_lock("1", &format!("{}/fstb/enable_switch_sync_flag", fpsgo_dir));
    write_lock("100", "/sys/module/mtk_fpsgo/parameters/uboost_enhance_f");
    write_lock("0", "/sys/module/mtk_fpsgo/parameters/isolation_limit_cap");

    // Pnpmgr & Perfmgr Boost
    write_lock("1", "/sys/pnpmgr/fpsgo_boost/boost_enable");
    write_lock("1", "/sys/pnpmgr/fpsgo_boost/boost_mode");
    write_lock("1", "/sys/pnpmgr/install");
    write_lock("1", "/proc/perfmgr/boost_ta/boost_ta_enable");
    write_lock("0", "/proc/perfmgr/smart/smart_enable");

    // GED Graphics Driver Boost
    write_lock("0", "/sys/module/sspm_v3/holders/ged/parameters/is_GED_KPI_enabled");
    write_lock("1", "/sys/module/ged/parameters/gx_game_mode");
    write_lock("1", "/sys/module/ged/parameters/gx_force_cpu_boost");
    write_lock("1", "/sys/module/ged/parameters/gx_boost_on");
    write_lock("1", "/sys/module/ged/parameters/enable_gpu_boost");
    write_lock("1", "/sys/module/ged/parameters/enable_cpu_boost");
    write_lock("1", "/sys/module/ged/parameters/ged_boost_enable");
    write_lock("1", "/sys/module/ged/parameters/boost_gpu_enable");
    write_lock("1", "/sys/module/ged/parameters/gpu_dvfs_enable");
    write_lock("1", "/sys/module/ged/parameters/gx_3D_benchmark_on");
    write_lock("100", "/sys/module/ged/parameters/boost_upper_bound");
    write_lock("100", "/sys/kernel/ged/hal/gpu_boost_level");
    write_lock("0", "/sys/module/ged/parameters/gpu_loading");
    write_lock("1", "/sys/module/ged/parameters/cpu_boost_policy");
    write_lock("1", "/sys/module/ged/parameters/boost_extra");

    // Comprehensive Touch Latency & Game Mode Hardware Boost
    write_lock("1", "/proc/perfmgr/touch_boost");
    write_lock("1", "/sys/module/perfmgr/parameters/perfmgr_enable");
    write_lock("1", "/sys/module/perfmgr_touch/parameters/touch_boost_enable");
    write_lock("150", "/sys/module/perfmgr_touch/parameters/touch_boost_duration_ms");

    // OEM MTK Touch Gaming Modes (Xiaomi, Oppo, Realme, Vivo, Transsion)
    write_lock("1", "/proc/touchpanel/game_switch_enable");
    write_lock("0", "/proc/touchpanel/oppo_tp_limit_enable");
    write_lock("0", "/proc/touchpanel/oppo_tp_direction");
    write_lock("1", "/proc/touchpanel/report_rate");
    write_lock("1", "/proc/touchpanel/touch_rate");
    write_lock("1", "/sys/class/touch/touch_dev/gesture_control");
    write_lock("1", "/sys/class/touch/touch_dev/game_mode");
    write_lock("1", "/sys/class/touch/touch_dev/touch_active");
    write_lock("1", "/sys/devices/virtual/touch/touch_dev/bump_sample_rate");
    write_lock("1", "/sys/touchscreen/game_mode");
    write_lock("1", "/sys/touchscreen/touch_active");
    write_lock("1", "/sys/devices/platform/tp_wake_switch/touch_boost");

    // Common Touch IC controller boost (Goodix, FocalTech, Novatek, Synaptics)
    let touch_patterns = [
        "/sys/devices/platform/*touch*/game_mode",
        "/sys/devices/platform/*touch*/boost",
        "/sys/devices/platform/*goodix*/*game*",
        "/sys/devices/platform/*goodix*/boost",
        "/sys/devices/platform/*focaltech*/*game*",
        "/sys/devices/platform/*novatek*/*game*",
        "/sys/devices/platform/*synaptics*/*game*",
        "/sys/class/input/input*/device/game_mode",
    ];
    for pattern in touch_patterns {
        if let Ok(paths) = glob::glob(pattern) {
            for path in paths.flatten() {
                if let Some(p_str) = path.to_str() {
                    write_lock("1", p_str);
                }
            }
        }
    }

    // 4. Memory & DRAM Boost
    write_lock("1", "/sys/devices/platform/boot_dramboost/dramboost/dramboost");

    // 5. GPU Tuning (Mali & PowerVR)
    if Path::new("/proc/gpufreq").exists() {
        if let Some(freq) = get_mtk_gpu_max_freq() {
            write_lock(&freq.to_string(), "/proc/gpufreq/gpufreq_opp_freq");
        }
        write_lock("0", "/proc/gpufreq/gpufreq_opp_idx");
    } else if Path::new("/proc/gpufreqv2").exists() {
        write_lock("0", "/proc/gpufreqv2/fix_target_opp_index");
    }

    if Path::new("/proc/gpufreq/gpufreq_power_limited").exists() {
        let settings = [
            "ignore_batt_oc", "ignore_batt_percent", "ignore_low_batt",
            "ignore_thermal_protect", "ignore_pbm_limited"
        ];
        for setting in settings {
            write_lock(&format!("{} 1", setting), "/proc/gpufreq/gpufreq_power_limited");
        }
    }

    let mali_patterns = [
        "/sys/devices/platform/*.mali",
        "/sys/devices/platform/soc/*.mali",
        "/sys/devices/platform/soc/*mali*",
    ];
    for pattern in mali_patterns {
        if let Ok(paths) = glob::glob(pattern) {
            for path in paths.flatten() {
                write_lock("always_on", &format!("{}/power_policy", path.display()));
            }
        }
    }

    if let Ok(paths) = glob::glob("/sys/devices/platform/soc/*mali*/scheduling/serialize_jobs") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("full", p_str);
            }
        }
    }

    if let Ok(paths) = glob::glob("/sys/devices/platform/soc/*mali*/js_ctx_scheduling_mode") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("1", p_str);
            }
        }
    }

    if let Ok(paths) = glob::glob("/sys/class/devfreq/*mali*/governor") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("performance", p_str);
            }
        }
    }

    if Path::new("/sys/module/pvrsrvkm/parameters/gpu_dvfs_enable").exists() {
        write_lock("0", "/sys/module/pvrsrvkm/parameters/gpu_dvfs_enable");
    }

    // 6. DVFSRC Maximum Performance OPP (Memory & VCore)
    write_lock("0", "/sys/kernel/helio-dvfsrc/dvfsrc_force_vcore_dvfs_opp");
    write_lock("performance", "/sys/class/devfreq/mtk-dvfsrc-devfreq/governor");

    if let Ok(paths) = glob::glob("/sys/devices/platform/*.dvfsrc") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("0", &format!("{}/helio-dvfsrc/dvfsrc_req_ddr_opp", p_str));
            }
        }
    }

    if let Ok(paths) = glob::glob("/sys/devices/platform/soc/*.dvfsrc") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("performance", &format!("{}/mtk-dvfsrc-devfreq/devfreq/mtk-dvfsrc-devfreq/governor", p_str));
            }
        }
    }

    // 7. Thermal & Safety Limiter Optimization for Gaming
    write_lock("1", "/proc/perfmgr/syslimiter/syslimiter_force_disable");
    write_lock("stop 1", "/proc/mtk_batoc_throttling/battery_oc_protect_stop");
    write_lock("0", "/sys/kernel/eara_thermal/enable");
    write_lock("0", "/proc/thermal/clatm_enable");
    write_lock("-1", "/proc/ppm/policy/pwr_thro_limit_cpu_freq");
    write_lock("-1", "/proc/ppm/policy/thermal_limit_cpu_freq");
}

/// Applies MediaTek-specific Powersave (ECO) Profile tuning
pub fn mediatek_powersave() {
    // 1. PPM (Processor Power Management)
    if Path::new("/proc/ppm/policy_status").exists() {
        let content = fs::read_to_string("/proc/ppm/policy_status").unwrap_or_default();
        for line in content.lines() {
            let is_target_1 = line.contains("FORCE_LIMIT") || line.contains("PWR_THRO") || line.contains("THERMAL") || line.contains("USER_LIMIT");
            let is_target_2 = line.contains("SYS_BOOST");
            if is_target_1 || is_target_2 {
                if let Some(idx_str) = line.split('[').nth(1).and_then(|s| s.split(']').next()) {
                    if is_target_1 {
                        write_lock(&format!("{} 1", idx_str), "/proc/ppm/policy_status");
                    }
                    if is_target_2 {
                        write_lock(&format!("{} 0", idx_str), "/proc/ppm/policy_status");
                    }
                }
            }
        }
    }
    ppm_fix_freq("-1");

    write_lock("1", "/proc/hps/enabled");
    write_lock("85", "/proc/hps/up_threshold");
    write_lock("70", "/proc/hps/down_threshold");
    write_lock("0", "/proc/hps/rush_boost_enabled");
    write_lock("0", "/proc/hps/rush_boost_threshold");

    // 2. CPUFreq & Memory
    write_lock("0", "/proc/cpufreq/cpufreq_cci_mode");
    write_lock("1", "/proc/cpufreq/cpufreq_power_mode");
    write_lock("0", "/sys/devices/platform/boot_dramboost/dramboost/dramboost");

    // 3. FPSGO & GED
    write_lock("2", "/sys/kernel/fpsgo/common/force_onoff");
    write_lock("1", "/sys/module/sspm_v3/holders/ged/parameters/is_GED_KPI_enabled");
    write_lock("0", "/sys/module/ged/parameters/gx_game_mode");
    write_lock("0", "/sys/module/ged/parameters/gx_force_cpu_boost");
    write_lock("0", "/sys/pnpmgr/fpsgo_boost/boost_enable");
    write_lock("0", "/sys/pnpmgr/fpsgo_boost/boost_mode");

    // 4. GPU Tuning (Mali & PowerVR)
    if Path::new("/proc/gpufreq").exists() {
        write_lock("0", "/proc/gpufreq/gpufreq_opp_freq");
    } else if Path::new("/proc/gpufreqv2").exists() {
        write_lock("-1", "/proc/gpufreqv2/fix_target_opp_index");
    }
    if Path::new("/sys/module/pvrsrvkm/parameters/gpu_dvfs_enable").exists() {
        write_lock("1", "/sys/module/pvrsrvkm/parameters/gpu_dvfs_enable");
    }
    if Path::new("/proc/gpufreq/gpufreq_power_limited").exists() {
        let settings = [
            "ignore_batt_oc", "ignore_batt_percent", "ignore_low_batt",
            "ignore_thermal_protect", "ignore_pbm_limited"
        ];
        for setting in settings {
            write_lock(&format!("{} 0", setting), "/proc/gpufreq/gpufreq_power_limited");
        }
    }

    if let Ok(mut paths) = glob::glob("/sys/devices/platform/*.mali") {
        if let Some(Ok(path)) = paths.next() {
            write_lock("coarse_demand", &format!("{}/power_policy", path.display()));
        }
    }
    if let Ok(paths) = glob::glob("/sys/class/devfreq/*mali*/governor") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("powersave", p_str);
            }
        }
    }

    // 5. DVFSRC Powersave OPP
    write_lock("-1", "/sys/kernel/helio-dvfsrc/dvfsrc_force_vcore_dvfs_opp");
    write_lock("powersave", "/sys/class/devfreq/mtk-dvfsrc-devfreq/governor");

    if let Ok(paths) = glob::glob("/sys/devices/platform/*.dvfsrc") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("-1", &format!("{}/helio-dvfsrc/dvfsrc_req_ddr_opp", p_str));
            }
        }
    }

    if let Ok(paths) = glob::glob("/sys/devices/platform/soc/*.dvfsrc") {
        for path in paths.flatten() {
            if let Some(p_str) = path.to_str() {
                write_lock("powersave", &format!("{}/mtk-dvfsrc-devfreq/devfreq/mtk-dvfsrc-devfreq/governor", p_str));
            }
        }
    }

    // 6. Thermal & Limits Protection
    write_lock("0", "/proc/perfmgr/syslimiter/syslimiter_force_disable");
    write_lock("stop 0", "/proc/mtk_batoc_throttling/battery_oc_protect_stop");
    write_lock("1", "/sys/kernel/eara_thermal/enable");
    write_lock("1", "/proc/thermal/clatm_enable");
}
