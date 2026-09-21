/*
 * Copyright (C) 2024-2025 Rem01Gaming
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
#include <stdlib.h>
#include <string.h>

static void boost_touch_input_pipeline(void);

/**
 * @brief Sets the maximum CPU nice priority (-20), real-time I/O priority,
 * and binds process and all its threads (RenderThread, Unity/UE worker threads,
 * Mali GPU driver threads) into top-app cpuset and schedtune.
 * @param pid The PID of the process to boost.
 */
void set_priority(const pid_t pid) {
    if (pid <= 0)
        return;

    // 1. Boost main process priority & I/O
    if (setpriority(PRIO_PROCESS, pid, -20) == -1)
        log_zenith(LOG_ERROR, "Unable to set nice priority for %d", pid);

    if (syscall(SYS_ioprio_set, 1, pid, (1 << 13) | 0) == -1)
        log_zenith(LOG_ERROR, "Unable to set IO priority for %d", pid);

    // Place main process into top-app cpusets (v1 and v2)
    write2file("/dev/cpuset/top-app/tasks", true, false, "%d\n", pid);
    write2file("/dev/cpuset/top-app/cgroup.procs", true, false, "%d\n", pid);
    write2file("/sys/fs/cgroup/top-app/cgroup.procs", true, false, "%d\n", pid);
    write2file("/dev/stune/top-app/tasks", true, false, "%d\n", pid);

    // 2. Iterate through all tasks/threads (RenderThread, GLES/Vulkan threads, WorkerThreads)
    char task_dir[64];
    snprintf(task_dir, sizeof(task_dir), "/proc/%d/task", pid);
    DIR* dir = opendir(task_dir);
    if (dir) {
        struct dirent* entry;
        while ((entry = readdir(dir)) != NULL) {
            if (entry->d_name[0] >= '0' && entry->d_name[0] <= '9') {
                pid_t tid = (pid_t)atoi(entry->d_name);
                if (tid > 0) {
                    setpriority(PRIO_PROCESS, tid, -20);
                    syscall(SYS_ioprio_set, 1, tid, (1 << 13) | 0);
                    write2file("/dev/cpuset/top-app/tasks", true, false, "%d\n", tid);
                    write2file("/dev/stune/top-app/tasks", true, false, "%d\n", tid);
                }
            }
        }
        closedir(dir);
    }

    // 3. Boost system touch input pipeline and pin touch IRQ to performance cores
    boost_touch_input_pipeline();
}

/**
 * @brief Boosts Android InputReader and InputDispatcher threads in system_server,
 * and pins touch panel hardware IRQs to performance cores for zero touch latency.
 */
static void boost_touch_input_pipeline(void) {
    // 1. Find system_server PID and boost input dispatcher & reader threads
    FILE* fp = popen("pidof system_server", "r");
    if (fp) {
        char buf[32] = {0};
        if (fgets(buf, sizeof(buf), fp)) {
            pid_t ss_pid = (pid_t)atoi(buf);
            if (ss_pid > 0) {
                char task_dir[64];
                snprintf(task_dir, sizeof(task_dir), "/proc/%d/task", ss_pid);
                DIR* dir = opendir(task_dir);
                if (dir) {
                    struct dirent* entry;
                    while ((entry = readdir(dir)) != NULL) {
                        if (entry->d_name[0] >= '0' && entry->d_name[0] <= '9') {
                            pid_t tid = (pid_t)atoi(entry->d_name);
                            if (tid > 0) {
                                char comm_path[80];
                                snprintf(comm_path, sizeof(comm_path), "/proc/%d/task/%d/comm", ss_pid, tid);
                                FILE* cf = fopen(comm_path, "r");
                                if (cf) {
                                    char comm[32] = {0};
                                    if (fgets(comm, sizeof(comm), cf)) {
                                        comm[strcspn(comm, "\r\n")] = '\0';
                                        if (strcmp(comm, "InputReader") == 0 ||
                                            strcmp(comm, "InputDispatcher") == 0 ||
                                            strcmp(comm, "SF-EventThread") == 0) {
                                            setpriority(PRIO_PROCESS, tid, -20);
                                            write2file("/dev/cpuset/top-app/tasks", true, false, "%d\n", tid);
                                            write2file("/dev/stune/top-app/tasks", true, false, "%d\n", tid);
                                        }
                                    }
                                    fclose(cf);
                                }
                            }
                        }
                    }
                    closedir(dir);
                }
            }
        }
        pclose(fp);
    }

    // 2. Pin touch controller interrupts (mtk-tpd, touchpanel, fts, goodix, etc.) to fast cores
    FILE* ifp = fopen("/proc/interrupts", "r");
    if (ifp) {
        char line[256];
        while (fgets(line, sizeof(line), ifp)) {
            if (strstr(line, "touch") || strstr(line, "tpd") || strstr(line, "fts") ||
                strstr(line, "goodix") || strstr(line, "synaptics") || strstr(line, "novatek")) {
                int irq = -1;
                if (sscanf(line, " %d:", &irq) == 1 && irq >= 0) {
                    char irq_path[64];
                    snprintf(irq_path, sizeof(irq_path), "/proc/irq/%d/smp_affinity_list", irq);
                    write2file(irq_path, false, false, "4-7\n");
                }
            }
        }
        fclose(ifp);
    }
}
