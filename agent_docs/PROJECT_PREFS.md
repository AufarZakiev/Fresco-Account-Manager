# BOINC Project Preferences & Remote Control Research

## Three Remote Channels (no companion daemon needed)

### 1. Account Manager Protocol (AM → BOINC Client)
Per-project `<account>` fields:
- `resource_share`, `suspend`, `dont_request_more_work`, `detach_when_done`
- `detach`, `update`, `abort_not_started`
- `no_rsc` (disable specific resource types: CPU, NVIDIA, ATI, intel_gpu)

Global:
- `global_preferences` (full override XML)
- `host_venue` (home/work/school)
- `repeat_sec` (sync interval)
- `no_project_notices`, `user_keywords`, `rss_feeds`

### 2. Project Web APIs (FAM → Project Server)
Standard endpoints (all BOINC projects):
- `get_project_config.php` — discover apps, platforms, plan classes (no auth)
- `create_account.php` / `lookup_account.php` — account management (no auth)
- `am_get_info.php?account_key=X` — read user profile + project_prefs XML
- `am_set_info.php?account_key=X` — write user profile + project_prefs XML
- `am_set_host_info.php?account_key=X&hostid=N&venue=V` — set host venue only
- `show_user.php?auth=X&format=xml` — user stats + host list

**IMPORTANT:** Use `web_rpc_url_base` from `get_project_config.php`, NOT `master_url`.
Example: climateprediction.net serves RPCs from `main.cpdn.org`.

### 3. Global Preferences (via AM reply `<global_preferences>`)
CPU: `max_ncpus_pct`, `cpu_usage_limit`, `cpu_scheduling_period_minutes`
Not-in-use variants: `niu_max_ncpus_pct`, `niu_cpu_usage_limit`, `niu_suspend_cpu_usage`
Schedule: `start_hour`/`end_hour`, `net_start_hour`/`net_end_hour`, per-day-of-week
Activity: `run_on_batteries`, `run_if_user_active`, `run_gpu_if_user_active`, `idle_time_to_run`, `suspend_cpu_usage`
Memory: `ram_max_used_busy_pct`, `ram_max_used_idle_pct`, `vm_max_used_pct`, `leave_apps_in_memory`
Disk: `disk_max_used_gb`, `disk_max_used_pct`, `disk_min_free_gb`
Network: `max_bytes_sec_up`, `max_bytes_sec_down`, `daily_xfer_limit_mb`
Work buffer: `work_buf_min_days`, `work_buf_additional_days`

---

## Project Preferences XML Format

### Standard fields (all projects)
```xml
<project_preferences>
    <resource_share>100</resource_share>
    <no_cpu>0</no_cpu>
    <no_cuda>0</no_cuda>
    <no_ati>0</no_ati>
    <no_intel_gpu>0</no_intel_gpu>
    <no_apple_gpu>0</no_apple_gpu>
    <allow_beta_work>0</allow_beta_work>
    <accelerate_gpu_apps>0</accelerate_gpu_apps>
    <apps_selected>
        <app_id>56</app_id>
    </apps_selected>
    <allow_non_preferred_apps>0</allow_non_preferred_apps>
    <project_specific>
        <!-- custom per project -->
    </project_specific>
    <venue name="home"><!-- same structure --></venue>
    <venue name="work"><!-- same structure --></venue>
    <venue name="school"><!-- same structure --></venue>
</project_preferences>
```

### BOINC framework feature flags (each project enables in project.inc)
- `APP_SELECT_PREFS` → `<apps_selected>` + `<app_id>` checkboxes
- `MAX_JOBS_PREF` → `<max_jobs>` max concurrent tasks
- `MAX_CPUS_PREF` → `<max_cpus>` max CPUs
- `COLOR_PREFS` → `<color_scheme>` screensaver colors
- `GFX_CPU_PREFS` → `<max_gfx_cpu_pct>` CPU limit for graphics
- `NON_GRAPHICAL_PREF` → `<non_graphical>` prefer non-graphical apps

### Per-project `<project_specific>` schemas

**Einstein@Home** — GPU utilization control:
```xml
<project_specific>
    <gpu_util_brp>0.500000</gpu_util_brp>   <!-- 0.5 = 2 BRP tasks/GPU -->
    <gpu_util_fgrp>1.000000</gpu_util_fgrp>
    <gpu_util_gw>1.000000</gpu_util_gw>
</project_specific>
```
Apps: einstein_O4AS(56), einstein_O4MD(59), einsteinbinary_BRP7(57),
      einsteinbinary_BRP4A(60), hsgamma_FGRP5(46), hsgamma_FGRPB1G(40),
      einsteinbinary_BRP4G(25), einsteinbinary_BRP4(19)

**PrimeGrid** — boolean subproject toggles (~25):
```xml
<project_specific>
    <llr321>1</llr321>
    <llrCUL>0</llrCUL>
    <genefer16>1</genefer16>
    <send_if_no_work>1</send_if_no_work>
</project_specific>
```
Custom "locations" system: 10 planet names + standard 3 venues.

**LHC@home** — experiment opt-in (SixTrack default, ATLAS/CMS/Theory must be enabled)

**Rosetta@home** — NO project_specific prefs. No app selection. Server-controlled.

**World Community Grid** — custom Device Profiles system, not standard BOINC prefs.
Account creation disabled via BOINC RPC — web registration only.

**GPUGRID** — GPU only (NVIDIA CUDA). Minimal prefs.

### Flow to update project_prefs via FAM
1. `am_get_info.php` — read current project_prefs XML
2. Parse, present as UI form
3. User edits
4. `am_set_info.php` — write updated XML back
5. `<update>1</update>` in next AM reply — force client scheduler contact

### Schema discovery strategies
- **Read-and-present**: fetch current prefs, infer form from existing XML
- **Curated registry**: maintain known schemas for popular projects
- No public schema file exists (`project_specific_prefs.inc` is server-side PHP)

---

## Companion Daemon — NOT needed for v1

AM protocol + project web APIs cover 90%+ of user needs:
- Attach/detach/suspend, app selection, GPU concurrency (via project_prefs)
- Resource share, disable GPU/CPU types, global computing preferences
- Host venue, beta work opt-in

Ship FAM without a daemon. Daemon is a v2 feature for power users.

### What only a daemon (GUI RPC) can do

| Feature | Who needs it |
|---|---|
| `exclude_gpu` by device number | Multi-GPU rigs: GPU 0→Project A, GPU 1→Project B |
| `app_config.xml` overrides | Projects that DON'T expose gpu_usage in prefs (unlike Einstein) |
| `process_priority` control | BOINC alongside latency-sensitive workloads |
| `exclusive_app`/`exclusive_gpu_app` | Pause BOINC when specific app launches (gaming) |
| `dont_use_vbox/wsl/docker` | Block specific runtimes |
| Real-time task suspend/resume/abort | Immediate individual task control |
| `set_run_mode` with duration | "Pause for 2 hours" commands |
| `ncpus` override | Override detected CPU count |

### Daemon candidate: Fresco Manager

Fresco Manager (BOINC Manager alternative, adjacent repo `../Fresco/`) already has:
- **Full GUI RPC implementation** in Rust (Tauri v2 + Vue 3 desktop app)
- TCP connection to port 31416 with MD5 challenge-response auth
- ~50+ RPC commands implemented (tasks, projects, modes, preferences, app_config, cc_config)
- **Remote connection mode** — can connect to any host:port, not just localhost
- CLI args: `--host`, `--port`, `--password`, `--datadir`, `--autostart`

Key source files:
- `../Fresco/src-tauri/src/lib.rs` (~1135 lines) — all Tauri commands, port 31416 constant
- `../Fresco/src-tauri/src/rpc/connection.rs` (~131 lines) — RpcClient struct, TCP connect, rpc_call
- `../Fresco/src-tauri/src/rpc/commands.rs` — high-level RPC wrappers
- `../Fresco/src-tauri/src/rpc/auth.rs` — MD5 challenge-response auth, gui_rpc_auth.cfg reading
- `../Fresco/src/composables/useRpc.ts` (~413 lines) — frontend RPC layer

Current data flow:
```
Vue 3 Frontend (WebView)
  ↓ Tauri IPC invoke()
Rust Backend (#[tauri::command] handlers)
  ↓ RpcClient.rpc_call() — XML wrapped in <boinc_gui_rpc_request>, 0x03 terminator
Local BOINC Client (127.0.0.1:31416)
```

To embed daemon functionality:
1. Add `--headless` mode (skip Tauri window creation)
2. Add HTTP/WebSocket listener for FAM server commands
3. Route incoming commands to existing RPC layer
4. No separate daemon install — users already have Fresco

Target architecture (v2):
```
FAM Server (cloud)
  ↓ HTTP/WebSocket
Fresco Manager (user's machine, --headless)
  ↓ TCP:31416 (already implemented)
Local BOINC Client
```

### Full GUI RPC-only feature list

**app_config.xml** (`set_app_config` RPC):
- `project_max_concurrent`, per-app `max_concurrent`
- Per-app `gpu_usage`/`cpu_usage` (BUT Einstein does this via project_prefs instead)
- Per-app-version `avg_ncpus`, `ngpus`, `cmdline`

**cc_config.xml** (`set_cc_config` RPC):
- `exclude_gpu` per project/device/type/app
- `ignore_nvidia_dev` / `ignore_ati_dev` / `ignore_intel_dev` per device number
- `ncpus` override, `no_gpus`, `no_opencl`
- `process_priority` / `process_priority_special`
- `exclusive_app` / `exclusive_gpu_app`
- `dont_use_vbox` / `dont_use_wsl` / `dont_use_docker`
- `max_file_xfers` / `max_file_xfers_per_project`
- `fetch_minimal_work`, `max_overdue_days`
- Full proxy configuration
- `device_name`, `suppress_net_info`

**Real-time operations:**
- `set_run_mode` / `set_gpu_mode` / `set_network_mode` with duration
- `suspend_result` / `resume_result` / `abort_result` (individual tasks)
- `abort_file_transfer` / `retry_file_transfer`
- `project_reset` (delete all project tasks & files)
- `run_benchmarks`, `quit` (shutdown client)

### GUI RPC requirements
- Direct TCP access to port 31416
- `--allow_remote_gui_rpc` or IP in `remote_hosts.cfg`
- Password from `gui_rpc_auth.cfg`
- Impractical over internet (NAT/firewall), designed for LAN

---

## Gotchas
- `am_set_host_info.php` only sets venue, despite the name
- Some projects disable client account creation (WCG) — use `lookup_account.php` only
- `no_*` fields are inverted booleans (1 = disabled)
- If `<apps_selected>` is empty/absent, ALL apps are selected (new apps auto-enrolled)
- Password hash format: `md5(password + lowercase(email))`
