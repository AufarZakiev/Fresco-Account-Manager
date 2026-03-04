<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { apiGetPreferences, apiSetPreferences } from "../api/client";
import {
  type GlobalPrefs,
  defaultGlobalPrefs,
  parseGlobalPrefs,
  serializeGlobalPrefs,
} from "../composables/useGlobalPrefsParser";

type Tab = "structured" | "advanced";

const activeTab = ref<Tab>("structured");
const prefs = ref<GlobalPrefs>(defaultGlobalPrefs());
const rawXml = ref("");
const modTime = ref("");
const loading = ref(true);
const saving = ref(false);
const error = ref("");
const success = ref("");

onMounted(async () => {
  try {
    const data = await apiGetPreferences();
    rawXml.value = data.prefs_xml;
    modTime.value = data.mod_time;
    if (data.prefs_xml) {
      prefs.value = parseGlobalPrefs(data.prefs_xml);
    }
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : "Failed to load preferences";
  } finally {
    loading.value = false;
  }
});

// Sync structured ↔ raw when switching tabs
watch(activeTab, (tab) => {
  if (tab === "advanced") {
    rawXml.value = serializeGlobalPrefs(prefs.value);
  } else {
    if (rawXml.value) {
      prefs.value = parseGlobalPrefs(rawXml.value);
    }
  }
});

function formatDate(dateStr: string): string {
  if (!dateStr) return "";
  return new Date(dateStr).toLocaleString();
}

async function handleSave() {
  saving.value = true;
  error.value = "";
  success.value = "";

  try {
    const xml =
      activeTab.value === "structured"
        ? serializeGlobalPrefs(prefs.value)
        : rawXml.value;

    await apiSetPreferences(xml);
    success.value = "Preferences saved successfully.";
    modTime.value = new Date().toISOString();

    rawXml.value = xml;
    prefs.value = parseGlobalPrefs(xml);
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : "Failed to save preferences";
  } finally {
    saving.value = false;
  }
}

function displayVal(field: keyof GlobalPrefs): string {
  const v = prefs.value[field];
  if (typeof v === "number") return v === 0 ? "" : String(v);
  return "";
}

function onNumInput(field: keyof GlobalPrefs, event: Event) {
  const raw = (event.target as HTMLInputElement).value.trim();
  if (raw === "") {
    (prefs.value[field] as number) = 0;
    return;
  }
  const num = Number(raw);
  if (!isNaN(num)) {
    (prefs.value[field] as number) = num;
  }
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1 class="page-title">Global Preferences</h1>
    </div>

    <p v-if="loading" class="muted">Loading...</p>

    <template v-else>
      <p v-if="error" class="error-banner">{{ error }}</p>
      <p v-if="success" class="success-banner">{{ success }}</p>

      <div class="prefs-tabs">
        <button
          :class="['prefs-tab', { active: activeTab === 'structured' }]"
          @click="activeTab = 'structured'"
        >
          Preferences
        </button>
        <button
          :class="['prefs-tab', { active: activeTab === 'advanced' }]"
          @click="activeTab = 'advanced'"
        >
          Advanced XML
        </button>
      </div>

      <!-- ═══ Structured form ═══ -->
      <div v-if="activeTab === 'structured'" class="prefs-sections">
        <div class="prefs-meta">
          <span class="text-xs muted">
            Last modified: {{ modTime ? formatDate(modTime) : "never" }}
          </span>
        </div>

        <!-- Computing Schedule -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Computing Schedule</h2>

          <label class="pref-row">
            <span>Run on batteries</span>
            <span
              class="toggle-switch"
              :class="{ on: prefs.runOnBatteries }"
              role="switch"
              :aria-checked="prefs.runOnBatteries"
              tabindex="0"
              @click.prevent="prefs.runOnBatteries = !prefs.runOnBatteries"
              @keydown.enter.prevent="prefs.runOnBatteries = !prefs.runOnBatteries"
              @keydown.space.prevent="prefs.runOnBatteries = !prefs.runOnBatteries"
            >
              <span class="toggle-knob" />
            </span>
          </label>

          <label class="pref-row">
            <span>Run if user is active</span>
            <span
              class="toggle-switch"
              :class="{ on: prefs.runIfUserActive }"
              role="switch"
              :aria-checked="prefs.runIfUserActive"
              tabindex="0"
              @click.prevent="prefs.runIfUserActive = !prefs.runIfUserActive"
              @keydown.enter.prevent="prefs.runIfUserActive = !prefs.runIfUserActive"
              @keydown.space.prevent="prefs.runIfUserActive = !prefs.runIfUserActive"
            >
              <span class="toggle-knob" />
            </span>
          </label>

          <label class="pref-row">
            <span>Run GPU if user is active</span>
            <span
              class="toggle-switch"
              :class="{ on: prefs.runGpuIfUserActive }"
              role="switch"
              :aria-checked="prefs.runGpuIfUserActive"
              tabindex="0"
              @click.prevent="prefs.runGpuIfUserActive = !prefs.runGpuIfUserActive"
              @keydown.enter.prevent="prefs.runGpuIfUserActive = !prefs.runGpuIfUserActive"
              @keydown.space.prevent="prefs.runGpuIfUserActive = !prefs.runGpuIfUserActive"
            >
              <span class="toggle-knob" />
            </span>
          </label>

          <label class="pref-row">
            <span>Idle time before computing (min)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('idleTimeToRun')"
              placeholder="No wait"
              @input="onNumInput('idleTimeToRun', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Start computing at (hour)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('startHour')"
              placeholder="All day"
              @input="onNumInput('startHour', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Stop computing at (hour)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('endHour')"
              placeholder="All day"
              @input="onNumInput('endHour', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Suspend when CPU usage above (%)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('suspendCpuUsage')"
              placeholder="Disabled"
              @input="onNumInput('suspendCpuUsage', $event)"
            />
          </label>
        </section>

        <!-- CPU -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">CPU</h2>

          <label class="pref-row">
            <span>Max CPUs used (%)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('maxNcpusPct')"
              placeholder="Use all"
              @input="onNumInput('maxNcpusPct', $event)"
            />
          </label>

          <label class="pref-row">
            <span>CPU usage limit (%)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('cpuUsageLimit')"
              placeholder="No limit"
              @input="onNumInput('cpuUsageLimit', $event)"
            />
          </label>
        </section>

        <!-- Memory -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Memory</h2>

          <label class="pref-row">
            <span>RAM when computer is busy (%)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('ramMaxUsedBusyPct')"
              placeholder="Use all"
              @input="onNumInput('ramMaxUsedBusyPct', $event)"
            />
          </label>

          <label class="pref-row">
            <span>RAM when computer is idle (%)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('ramMaxUsedIdlePct')"
              placeholder="Use all"
              @input="onNumInput('ramMaxUsedIdlePct', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Leave apps in memory while suspended</span>
            <span
              class="toggle-switch"
              :class="{ on: prefs.leaveAppsInMemory }"
              role="switch"
              :aria-checked="prefs.leaveAppsInMemory"
              tabindex="0"
              @click.prevent="prefs.leaveAppsInMemory = !prefs.leaveAppsInMemory"
              @keydown.enter.prevent="prefs.leaveAppsInMemory = !prefs.leaveAppsInMemory"
              @keydown.space.prevent="prefs.leaveAppsInMemory = !prefs.leaveAppsInMemory"
            >
              <span class="toggle-knob" />
            </span>
          </label>
        </section>

        <!-- Disk -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Disk</h2>

          <label class="pref-row">
            <span>Max disk space (GB)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('diskMaxUsedGb')"
              placeholder="No limit"
              @input="onNumInput('diskMaxUsedGb', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Min free disk space (GB)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('diskMinFreeGb')"
              placeholder="No limit"
              @input="onNumInput('diskMinFreeGb', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Max disk usage (%)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('diskMaxUsedPct')"
              placeholder="No limit"
              @input="onNumInput('diskMaxUsedPct', $event)"
            />
          </label>
        </section>

        <!-- Network -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Network</h2>

          <label class="pref-row">
            <span>Max upload rate (KB/s)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('maxBytesSecUp')"
              placeholder="No limit"
              @input="onNumInput('maxBytesSecUp', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Max download rate (KB/s)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('maxBytesSecDown')"
              placeholder="No limit"
              @input="onNumInput('maxBytesSecDown', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Daily transfer limit (MB)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('dailyXferLimitMb')"
              placeholder="No limit"
              @input="onNumInput('dailyXferLimitMb', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Network start hour</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('netStartHour')"
              placeholder="All day"
              @input="onNumInput('netStartHour', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Network end hour</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('netEndHour')"
              placeholder="All day"
              @input="onNumInput('netEndHour', $event)"
            />
          </label>
        </section>

        <!-- Work Buffer -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Work Buffer</h2>

          <label class="pref-row">
            <span>Minimum work buffer (days)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('workBufMinDays')"
              placeholder="Default"
              @input="onNumInput('workBufMinDays', $event)"
            />
          </label>

          <label class="pref-row">
            <span>Additional work buffer (days)</span>
            <input
              type="text"
              inputmode="decimal"
              class="pref-input"
              :value="displayVal('workBufAdditionalDays')"
              placeholder="Default"
              @input="onNumInput('workBufAdditionalDays', $event)"
            />
          </label>
        </section>

        <!-- Phase 2 disabled section -->
        <section class="card prefs-section phase2-section">
          <h2 class="prefs-section-title">
            Advanced Host Controls
            <span class="phase2-badge">Requires Fresco</span>
          </h2>

          <label class="pref-row">
            <span>GPU device exclusions</span>
            <input
              type="text"
              class="pref-input"
              disabled
              placeholder="Per-device GPU assignment"
            />
          </label>

          <label class="pref-row">
            <span>CPU count override</span>
            <input
              type="text"
              class="pref-input"
              disabled
              placeholder="Auto"
            />
          </label>

          <label class="pref-row">
            <span>Process priority</span>
            <select class="pref-select" disabled>
              <option>Normal</option>
            </select>
          </label>

          <label class="pref-row">
            <span>Don't use VirtualBox</span>
            <span class="toggle-switch disabled" role="switch" aria-checked="false">
              <span class="toggle-knob" />
            </span>
          </label>

          <label class="pref-row">
            <span>Don't use WSL</span>
            <span class="toggle-switch disabled" role="switch" aria-checked="false">
              <span class="toggle-knob" />
            </span>
          </label>

          <label class="pref-row">
            <span>Don't use Docker</span>
            <span class="toggle-switch disabled" role="switch" aria-checked="false">
              <span class="toggle-knob" />
            </span>
          </label>

          <label class="pref-row">
            <span>Exclusive apps</span>
            <input
              type="text"
              class="pref-input"
              disabled
              placeholder="Not available"
            />
          </label>
        </section>

        <div class="prefs-actions">
          <button
            class="btn-primary"
            :disabled="saving"
            @click="handleSave"
          >
            {{ saving ? "Saving..." : "Save Preferences" }}
          </button>
        </div>
      </div>

      <!-- ═══ Advanced XML tab ═══ -->
      <div v-else class="card prefs-card">
        <div class="prefs-meta">
          <span class="text-xs muted">
            Last modified: {{ modTime ? formatDate(modTime) : "never" }}
          </span>
        </div>

        <label class="prefs-label">
          Preferences XML
          <textarea
            v-model="rawXml"
            class="prefs-textarea"
            rows="20"
            spellcheck="false"
          ></textarea>
        </label>

        <div class="prefs-actions">
          <button
            class="btn-primary"
            :disabled="saving"
            @click="handleSave"
          >
            {{ saving ? "Saving..." : "Save" }}
          </button>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.prefs-tabs {
  display: flex;
  gap: 0;
  margin-bottom: var(--space-lg);
  border-bottom: 1px solid var(--color-border);
}

.prefs-tab {
  padding: var(--space-sm) var(--space-lg);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.prefs-tab:hover {
  color: var(--color-text-primary);
}

.prefs-tab.active {
  color: var(--color-accent);
  border-bottom-color: var(--color-accent);
}

.prefs-sections {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.prefs-section {
  padding: var(--space-md) var(--space-lg);
}

.prefs-section-title {
  font-size: var(--font-size-base);
  font-weight: 600;
  margin-bottom: var(--space-sm);
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

/* ── Fresco-style pref rows ── */

.pref-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 42px;
  padding: 0;
  border-bottom: 1px solid var(--color-border-light);
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
  cursor: default;
  margin-bottom: 0;
  font-weight: normal;
}

.pref-row:last-child {
  border-bottom: none;
}

.pref-input {
  width: min(130px, 40vw);
  padding: 5px 8px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
  text-align: right;
  background: var(--color-bg);
  color: var(--color-text-primary);
  transition: border-color var(--transition-fast);
}

.pref-input:focus {
  outline: none;
  border-color: var(--color-accent);
}

.pref-input::placeholder {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
}

.pref-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pref-select {
  width: min(130px, 40vw);
  padding: 5px 8px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg);
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
}

.pref-select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ── Meta / actions ── */

.prefs-meta {
  display: flex;
  justify-content: flex-end;
}

.prefs-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: var(--space-sm);
}

/* ── Advanced XML tab ── */

.prefs-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.prefs-label {
  margin-bottom: 0;
}

.prefs-textarea {
  margin-top: var(--space-sm);
  font-family: "Cascadia Code", "Fira Code", Consolas, monospace;
  font-size: var(--font-size-sm);
  line-height: 1.5;
  resize: vertical;
  min-height: 200px;
}

/* ── Phase 2 disabled ── */

.phase2-section {
  opacity: 0.45;
  pointer-events: none;
  position: relative;
}

.phase2-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: var(--font-size-xs);
  font-weight: 500;
  background: var(--color-warning-light);
  color: #92400e;
}
</style>
