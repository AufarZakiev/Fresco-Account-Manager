<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
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

// Sync structured → raw when switching to advanced tab
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

    // Keep both representations in sync
    rawXml.value = xml;
    prefs.value = parseGlobalPrefs(xml);
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : "Failed to save preferences";
  } finally {
    saving.value = false;
  }
}

const cpuPctLabel = computed(() => `${prefs.value.maxNcpusPct}%`);
const cpuUsageLabel = computed(() => `${prefs.value.cpuUsageLimit}%`);
const ramBusyLabel = computed(() => `${prefs.value.ramMaxUsedBusyPct}%`);
const ramIdleLabel = computed(() => `${prefs.value.ramMaxUsedIdlePct}%`);
const diskPctLabel = computed(() => `${prefs.value.diskMaxUsedPct}%`);
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
          <div class="prefs-grid">
            <label class="switch-label">
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
              Run on batteries
            </label>
            <label class="switch-label">
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
              Run if user is active
            </label>
            <label class="switch-label">
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
              Run GPU if user is active
            </label>

            <label class="field-label">
              Idle time before computing (minutes)
              <input
                type="number"
                v-model.number="prefs.idleTimeToRun"
                min="0"
                step="1"
              />
            </label>
            <label class="field-label">
              Start computing at (hour, 0-24)
              <input
                type="number"
                v-model.number="prefs.startHour"
                min="0"
                max="24"
                step="0.5"
              />
            </label>
            <label class="field-label">
              Stop computing at (hour, 0-24)
              <input
                type="number"
                v-model.number="prefs.endHour"
                min="0"
                max="24"
                step="0.5"
              />
            </label>
            <label class="field-label">
              Suspend when CPU usage above (%)
              <input
                type="number"
                v-model.number="prefs.suspendCpuUsage"
                min="0"
                max="100"
              />
            </label>
          </div>
        </section>

        <!-- CPU -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">CPU</h2>
          <div class="prefs-grid">
            <label class="range-label">
              Max CPUs used: <strong>{{ cpuPctLabel }}</strong>
              <input
                type="range"
                v-model.number="prefs.maxNcpusPct"
                min="0"
                max="100"
                step="5"
              />
            </label>
            <label class="range-label">
              CPU usage limit: <strong>{{ cpuUsageLabel }}</strong>
              <input
                type="range"
                v-model.number="prefs.cpuUsageLimit"
                min="0"
                max="100"
                step="5"
              />
            </label>
          </div>
        </section>

        <!-- Memory -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Memory</h2>
          <div class="prefs-grid">
            <label class="range-label">
              RAM when busy: <strong>{{ ramBusyLabel }}</strong>
              <input
                type="range"
                v-model.number="prefs.ramMaxUsedBusyPct"
                min="0"
                max="100"
                step="5"
              />
            </label>
            <label class="range-label">
              RAM when idle: <strong>{{ ramIdleLabel }}</strong>
              <input
                type="range"
                v-model.number="prefs.ramMaxUsedIdlePct"
                min="0"
                max="100"
                step="5"
              />
            </label>
            <label class="switch-label">
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
              Leave apps in memory while suspended
            </label>
          </div>
        </section>

        <!-- Disk -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Disk</h2>
          <div class="prefs-grid">
            <label class="field-label">
              Max disk space (GB, 0 = no limit)
              <input
                type="number"
                v-model.number="prefs.diskMaxUsedGb"
                min="0"
                step="1"
              />
            </label>
            <label class="field-label">
              Min free disk space (GB)
              <input
                type="number"
                v-model.number="prefs.diskMinFreeGb"
                min="0"
                step="0.1"
              />
            </label>
            <label class="range-label">
              Max disk usage: <strong>{{ diskPctLabel }}</strong>
              <input
                type="range"
                v-model.number="prefs.diskMaxUsedPct"
                min="0"
                max="100"
                step="5"
              />
            </label>
          </div>
        </section>

        <!-- Network -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Network</h2>
          <div class="prefs-grid">
            <label class="field-label">
              Max upload rate (KB/s, 0 = no limit)
              <input
                type="number"
                v-model.number="prefs.maxBytesSecUp"
                min="0"
                step="1"
              />
            </label>
            <label class="field-label">
              Max download rate (KB/s, 0 = no limit)
              <input
                type="number"
                v-model.number="prefs.maxBytesSecDown"
                min="0"
                step="1"
              />
            </label>
            <label class="field-label">
              Daily transfer limit (MB, 0 = no limit)
              <input
                type="number"
                v-model.number="prefs.dailyXferLimitMb"
                min="0"
                step="1"
              />
            </label>
            <label class="field-label">
              Network start hour (0-24)
              <input
                type="number"
                v-model.number="prefs.netStartHour"
                min="0"
                max="24"
                step="0.5"
              />
            </label>
            <label class="field-label">
              Network end hour (0-24)
              <input
                type="number"
                v-model.number="prefs.netEndHour"
                min="0"
                max="24"
                step="0.5"
              />
            </label>
          </div>
        </section>

        <!-- Work Buffer -->
        <section class="card prefs-section">
          <h2 class="prefs-section-title">Work Buffer</h2>
          <div class="prefs-grid">
            <label class="field-label">
              Minimum work buffer (days)
              <input
                type="number"
                v-model.number="prefs.workBufMinDays"
                min="0"
                step="0.1"
              />
            </label>
            <label class="field-label">
              Additional work buffer (days)
              <input
                type="number"
                v-model.number="prefs.workBufAdditionalDays"
                min="0"
                step="0.1"
              />
            </label>
          </div>
        </section>

        <!-- Phase 2 disabled section -->
        <section class="card prefs-section phase2-section">
          <h2 class="prefs-section-title">
            Advanced Host Controls
            <span class="phase2-badge">Requires Fresco</span>
          </h2>
          <div class="prefs-grid">
            <label class="field-label">
              GPU device exclusions
              <input type="text" disabled placeholder="Per-device GPU assignment" />
            </label>
            <label class="field-label">
              CPU count override
              <input type="number" disabled placeholder="Auto" />
            </label>
            <label class="field-label">
              Process priority
              <select disabled>
                <option>Normal</option>
              </select>
            </label>
            <label class="switch-label">
              <span class="toggle-switch disabled" role="switch" aria-checked="false">
                <span class="toggle-knob" />
              </span>
              Don't use VirtualBox
            </label>
            <label class="switch-label">
              <span class="toggle-switch disabled" role="switch" aria-checked="false">
                <span class="toggle-knob" />
              </span>
              Don't use WSL
            </label>
            <label class="switch-label">
              <span class="toggle-switch disabled" role="switch" aria-checked="false">
                <span class="toggle-knob" />
              </span>
              Don't use Docker
            </label>
            <label class="field-label">
              Exclusive apps
              <input type="text" disabled placeholder="Not available" />
            </label>
          </div>
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
  margin-bottom: var(--space-md);
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.prefs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: var(--space-md) var(--space-lg);
  align-items: start;
}

.field-label {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  font-weight: 500;
  margin-bottom: 0;
}

.switch-label {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  font-weight: 500;
  cursor: pointer;
  user-select: none;
  margin-bottom: 0;
}

.field-label input,
.field-label select {
  margin-top: 0;
}

.prefs-meta {
  display: flex;
  justify-content: flex-end;
}

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

.prefs-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: var(--space-sm);
}

/* Phase 2 disabled styling */
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
