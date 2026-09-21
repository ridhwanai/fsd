/*
 * Copyright (C) 2026-2027 Zexshia
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include <AZenith.h>
#include <dirent.h>
#include <string.h>

/*
 * Bypass charging node definitions for MediaTek (MTK) platforms
 * Includes MTK kernel sysfs, Transsion MTK, OPlus MTK, and Xiaomi MTK nodes.
 */
BypassNode bypass_list[] = {
    // 1. Generic / Common Power Supply Nodes used across MTK devices
    {"COMMON_INPUT_SUSPEND", "/sys/class/power_supply/battery/input_suspend", "1", "0"},
    {"COMMON_BATT_INPUT_SUSPEND", "/sys/class/power_supply/battery/battery_input_suspend", "1", "0"},
    {"COMMON_CHG_CONTROL", "/sys/class/power_supply/battery/charger_control", "0", "1"},
    {"COMMON_CHG_DISABLE", "/sys/class/power_supply/battery/charge_disable", "1", "0"},
    {"COMMON_CHG_ENABLED_V1", "/sys/class/power_supply/battery/charging_enabled", "0", "1"},
    {"COMMON_CHG_ENABLED_V2", "/sys/class/power_supply/battery/charge_enabled", "0", "1"},
    {"COMMON_BATT_CHG_ENABLED", "/sys/class/power_supply/battery/battery_charging_enabled", "0", "1"},
    {"COMMON_DEVICE_CHG_EN", "/sys/class/power_supply/battery/device/Charging_Enable", "0", "1"},
    {"AC_CHG_ENABLED", "/sys/class/power_supply/ac/charging_enabled", "0", "1"},
    {"CHG_DATA_ENABLE", "/sys/class/power_supply/charge_data/enable_charger", "0", "1"},
    {"DC_CHG_ENABLED", "/sys/class/power_supply/dc/charging_enabled", "0", "1"},
    {"BATT_CONNECT_DISABLE", "/sys/class/power_supply/battery/connect_disable", "1", "0"},
    {"CHGALG_DISABLE_CHG", "/sys/class/power_supply/chargalg/disable_charging", "1", "0"},
    {"OP_DISABLE_CHG", "/sys/class/power_supply/battery/op_disable_charge", "1", "0"},

    // 2. MediaTek Native Kernel Charger & Battery Driver Nodes
    {"MTK_BYPASS_CHG", "/sys/devices/platform/charger/bypass_charger", "1", "0"},
    {"MTK_CURRENT_CMD", "/proc/mtk_battery_cmd/current_cmd", "0 1", "0 0"},
    {"MTK_DISABLE_BATTERY_CHG", "/sys/devices/platform/mt-battery/disable_charger", "1", "0"},
    {"MTK_ADV_PATH", "/proc/mtk_battery_cmd/en_power_path", "0", "1"},

    // 3. Transsion (Infinix / Tecno) MediaTek Nodes
    {"TRAN_AICHG_DISABLE", "/sys/devices/platform/charger/tran_aichg_disable_charger", "1", "0"},

    // 4. OPlus (OPPO / Realme) MediaTek Charging Nodes
    {"OPLUS_MMI_1", "/sys/class/oplus_chg/battery/mmi_charging_enable", "0", "1"},
    {"OPLUS_MMI_2", "/sys/class/power_supply/battery/mmi_charging_enable", "0", "1"},
    {"OPLUS_MMI_3", "/sys/devices/virtual/oplus_chg/battery/mmi_charging_enable", "0", "1"},
    {"OPLUS_MMI_SOC", "/sys/devices/platform/soc/soc:oplus,chg_intf/oplus_chg/battery/mmi_charging_enable", "0", "1"},
    {"OPLUS_EXP_CHG_ENABLE", "/sys/devices/platform/soc/soc:oplus,chg_intf/oplus_chg/battery/chg_enable", "0", "1"},
    {"OPLUS_COOLDOWN_STATE", "/sys/devices/platform/soc/soc:oplus,chg_intf/oplus_chg/battery/cool_down", "1", "0"},

    // 5. Xiaomi (Redmi / POCO) MediaTek Charging Nodes
    {"XIAOMI_MCA_INPUT_SUSPEND_V1", "/sys/devices/platform/soc/soc:mca_charge_interface/input_suspend", "1", "0"},
    {"XIAOMI_MCA_CHARGE_ENABLE_V1", "/sys/devices/platform/soc/soc:mca_charge_interface/charge_enable", "0", "1"},
    {"XIAOMI_MCA_STOP_HANDLE_V1", "/sys/devices/platform/soc/soc:mca_business_charger/stop_handle_charge", "1", "0"},
    {"XIAOMI_XM_INPUT_SUSPEND", "/sys/class/xm_power/charger/charge_interface/input_suspend", "1", "0"},
    {"XIAOMI_XM_STOP_HANDLE", "/sys/class/xm_power/charger/charger_common/stop_handle_charge", "1", "0"},
    {"XIAOMI_XM_CHARGE_ENABLE", "/sys/class/xm_power/charger/charge_interface/charge_enable", "0", "1"},

    // 6. Generic MediaTek Charger Hardware Controls
    {"CHG_LIMIT_ENABLE", "/proc/driver/charger_limit_enable", "1", "0"},
    {"CHG_LIMIT_VAL", "/proc/driver/charger_limit", "5", "100"},
    {"ADAPTER_CC_MODE", "/sys/class/power_supply/main/adapter_cc_mode", "1", "0"},
    {"COOL_MODE_MAIN", "/sys/class/power_supply/main/cool_mode", "1", "0"},
    {"RESTRICTED_CHG_BATT", "/sys/class/power_supply/battery/restricted_charging", "1", "0"}
};

const int bypass_list_size = sizeof(bypass_list) / sizeof(BypassNode);
