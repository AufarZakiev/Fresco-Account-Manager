/**
 * Converts between BOINC global preferences XML and a structured object.
 *
 * Only handles the subset of fields that FAM's UI exposes.
 * Unknown tags are preserved in `extraXml` for round-trip fidelity.
 */

export interface GlobalPrefs {
  // Computing schedule
  runOnBatteries: boolean;
  runIfUserActive: boolean;
  runGpuIfUserActive: boolean;
  idleTimeToRun: number;
  startHour: number;
  endHour: number;
  suspendCpuUsage: number;

  // CPU
  maxNcpusPct: number;
  cpuUsageLimit: number;

  // Memory
  ramMaxUsedBusyPct: number;
  ramMaxUsedIdlePct: number;
  leaveAppsInMemory: boolean;

  // Disk
  diskMaxUsedGb: number;
  diskMinFreeGb: number;
  diskMaxUsedPct: number;

  // Network
  maxBytesSecUp: number;
  maxBytesSecDown: number;
  dailyXferLimitMb: number;
  netStartHour: number;
  netEndHour: number;

  // Work buffer
  workBufMinDays: number;
  workBufAdditionalDays: number;

  // Preserved verbatim (tags we don't have UI for)
  extraXml: string;
}

const FIELD_MAP: Record<string, { key: keyof GlobalPrefs; type: "bool" | "num" }> = {
  run_on_batteries: { key: "runOnBatteries", type: "bool" },
  run_if_user_active: { key: "runIfUserActive", type: "bool" },
  run_gpu_if_user_active: { key: "runGpuIfUserActive", type: "bool" },
  idle_time_to_run: { key: "idleTimeToRun", type: "num" },
  start_hour: { key: "startHour", type: "num" },
  end_hour: { key: "endHour", type: "num" },
  suspend_cpu_usage: { key: "suspendCpuUsage", type: "num" },
  max_ncpus_pct: { key: "maxNcpusPct", type: "num" },
  cpu_usage_limit: { key: "cpuUsageLimit", type: "num" },
  ram_max_used_busy_pct: { key: "ramMaxUsedBusyPct", type: "num" },
  ram_max_used_idle_pct: { key: "ramMaxUsedIdlePct", type: "num" },
  leave_apps_in_memory: { key: "leaveAppsInMemory", type: "bool" },
  disk_max_used_gb: { key: "diskMaxUsedGb", type: "num" },
  disk_min_free_gb: { key: "diskMinFreeGb", type: "num" },
  disk_max_used_pct: { key: "diskMaxUsedPct", type: "num" },
  max_bytes_sec_up: { key: "maxBytesSecUp", type: "num" },
  max_bytes_sec_down: { key: "maxBytesSecDown", type: "num" },
  daily_xfer_limit_mb: { key: "dailyXferLimitMb", type: "num" },
  net_start_hour: { key: "netStartHour", type: "num" },
  net_end_hour: { key: "netEndHour", type: "num" },
  work_buf_min_days: { key: "workBufMinDays", type: "num" },
  work_buf_additional_days: { key: "workBufAdditionalDays", type: "num" },
};

const KNOWN_TAGS = new Set(Object.keys(FIELD_MAP));
// Also skip mod_time — we handle it separately
KNOWN_TAGS.add("mod_time");

export function defaultGlobalPrefs(): GlobalPrefs {
  return {
    runOnBatteries: false,
    runIfUserActive: true,
    runGpuIfUserActive: false,
    idleTimeToRun: 3,
    startHour: 0,
    endHour: 0,
    suspendCpuUsage: 0,
    maxNcpusPct: 100,
    cpuUsageLimit: 100,
    ramMaxUsedBusyPct: 50,
    ramMaxUsedIdlePct: 90,
    leaveAppsInMemory: false,
    diskMaxUsedGb: 0,
    diskMinFreeGb: 0.1,
    diskMaxUsedPct: 90,
    maxBytesSecUp: 0,
    maxBytesSecDown: 0,
    dailyXferLimitMb: 0,
    netStartHour: 0,
    netEndHour: 0,
    workBufMinDays: 0.1,
    workBufAdditionalDays: 0.25,
    extraXml: "",
  };
}

/**
 * Parse global_preferences XML (the inner content, without the wrapping tag)
 * into a structured object.
 */
export function parseGlobalPrefs(xml: string): GlobalPrefs {
  const prefs = defaultGlobalPrefs();
  const extraLines: string[] = [];

  // Strip outer <global_preferences> tag if present
  const inner = stripOuterTag(xml, "global_preferences");

  // Simple line-by-line tag parser — works for BOINC's flat XML structure
  const tagRegex = /^\s*<(\w+)>(.*?)<\/\1>\s*$/;
  for (const line of inner.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("<?")) continue;

    const match = trimmed.match(tagRegex);
    if (!match) {
      // Preserve non-matching lines (comments, multi-line blocks, etc.)
      if (trimmed.length > 0) {
        extraLines.push(line);
      }
      continue;
    }

    const [, tagName, value] = match;
    const mapping = FIELD_MAP[tagName];
    if (mapping) {
      if (mapping.type === "bool") {
        (prefs[mapping.key] as boolean) = value === "1" || value.toLowerCase() === "true";
      } else {
        (prefs[mapping.key] as number) = parseFloat(value) || 0;
      }
    } else if (!KNOWN_TAGS.has(tagName)) {
      extraLines.push(line);
    }
  }

  prefs.extraXml = extraLines.join("\n");
  return prefs;
}

/**
 * Serialize a structured prefs object back to global_preferences XML.
 */
export function serializeGlobalPrefs(prefs: GlobalPrefs): string {
  const lines: string[] = [];
  lines.push("<global_preferences>");

  const emit = (tag: string, value: string) => {
    lines.push(`  <${tag}>${value}</${tag}>`);
  };

  const emitBool = (tag: string, key: keyof GlobalPrefs) => {
    emit(tag, (prefs[key] as boolean) ? "1" : "0");
  };

  const emitNum = (tag: string, key: keyof GlobalPrefs) => {
    const val = prefs[key] as number;
    // Use integer format if whole number, otherwise 6 decimal places max
    emit(tag, Number.isInteger(val) ? val.toString() : val.toFixed(6).replace(/\.?0+$/, ""));
  };

  // Timestamp
  emit("mod_time", Math.floor(Date.now() / 1000).toString());

  // Computing schedule
  emitBool("run_on_batteries", "runOnBatteries");
  emitBool("run_if_user_active", "runIfUserActive");
  emitBool("run_gpu_if_user_active", "runGpuIfUserActive");
  emitNum("idle_time_to_run", "idleTimeToRun");
  emitNum("start_hour", "startHour");
  emitNum("end_hour", "endHour");
  emitNum("suspend_cpu_usage", "suspendCpuUsage");

  // CPU
  emitNum("max_ncpus_pct", "maxNcpusPct");
  emitNum("cpu_usage_limit", "cpuUsageLimit");

  // Memory
  emitNum("ram_max_used_busy_pct", "ramMaxUsedBusyPct");
  emitNum("ram_max_used_idle_pct", "ramMaxUsedIdlePct");
  emitBool("leave_apps_in_memory", "leaveAppsInMemory");

  // Disk
  emitNum("disk_max_used_gb", "diskMaxUsedGb");
  emitNum("disk_min_free_gb", "diskMinFreeGb");
  emitNum("disk_max_used_pct", "diskMaxUsedPct");

  // Network
  emitNum("max_bytes_sec_up", "maxBytesSecUp");
  emitNum("max_bytes_sec_down", "maxBytesSecDown");
  emitNum("daily_xfer_limit_mb", "dailyXferLimitMb");
  emitNum("net_start_hour", "netStartHour");
  emitNum("net_end_hour", "netEndHour");

  // Work buffer
  emitNum("work_buf_min_days", "workBufMinDays");
  emitNum("work_buf_additional_days", "workBufAdditionalDays");

  // Preserved extra XML
  if (prefs.extraXml.trim()) {
    lines.push(prefs.extraXml);
  }

  lines.push("</global_preferences>");
  return lines.join("\n");
}

function stripOuterTag(xml: string, tagName: string): string {
  const openTag = `<${tagName}>`;
  const closeTag = `</${tagName}>`;
  let s = xml.trim();

  // Handle opening tag (possibly with attributes)
  const openIdx = s.indexOf(`<${tagName}`);
  if (openIdx !== -1) {
    const endOfOpen = s.indexOf(">", openIdx);
    if (endOfOpen !== -1) {
      s = s.substring(endOfOpen + 1);
    }
  }

  const closeIdx = s.lastIndexOf(closeTag);
  if (closeIdx !== -1) {
    s = s.substring(0, closeIdx);
  }

  return s;
}
