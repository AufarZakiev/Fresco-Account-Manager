<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { useProjectsStore } from "../stores/projects";
import {
  apiSuspendProject,
  apiResumeProject,
  apiDetachProject,
  apiGetProjectPrefs,
  apiSetProjectPrefs,
  type UserProject,
} from "../api/client";

const store = useProjectsStore();

const savingId = ref<number | null>(null);
const leavingId = ref<number | null>(null);
const detachingId = ref<number | null>(null);

// Project prefs panel state (keyed by user_project id)
const prefsOpen = reactive<Record<number, boolean>>({});
const prefsLoading = reactive<Record<number, boolean>>({});
const prefsData = reactive<Record<number, string>>({});
const prefsSaving = reactive<Record<number, boolean>>({});
const prefsError = reactive<Record<number, string>>({});
const prefsSuccess = reactive<Record<number, string>>({});

const RESOURCE_TYPES = [
  { value: "CPU", label: "CPU" },
  { value: "NVIDIA", label: "NVIDIA GPU" },
  { value: "ATI", label: "ATI/AMD GPU" },
  { value: "intel_gpu", label: "Intel GPU" },
];

onMounted(async () => {
  await store.fetchUserProjects();
});

async function toggleSuspend(up: UserProject) {
  savingId.value = up.id;
  if (up.suspended) {
    await apiResumeProject(up.id);
  } else {
    await apiSuspendProject(up.id);
  }
  await store.fetchUserProjects();
  savingId.value = null;
}

async function toggleDontRequestMoreWork(up: UserProject) {
  savingId.value = up.id;
  await store.updateProject(up.id, {
    dont_request_more_work: !up.dont_request_more_work,
  });
  savingId.value = null;
}

async function saveResourceShare(upId: number, value: number) {
  savingId.value = upId;
  await store.updateProject(upId, { resource_share: value });
  savingId.value = null;
}

async function toggleNoRsc(up: UserProject, rscType: string) {
  savingId.value = up.id;
  const current = up.no_rsc || [];
  const newNoRsc = current.includes(rscType)
    ? current.filter((r) => r !== rscType)
    : [...current, rscType];
  await store.updateProject(up.id, { no_rsc: newNoRsc });
  savingId.value = null;
}

async function detach(upId: number) {
  if (
    !confirm(
      "Detach this project? The BOINC client will detach on the next sync.",
    )
  )
    return;
  detachingId.value = upId;
  await apiDetachProject(upId);
  await store.fetchUserProjects();
  detachingId.value = null;
}

async function leave(upId: number) {
  if (!confirm("Remove this project immediately? You can re-join later."))
    return;
  leavingId.value = upId;
  await store.leave(upId);
  leavingId.value = null;
}

function onShareChange(upId: number, event: Event) {
  const target = event.target as HTMLInputElement;
  const value = Number(target.value);
  saveResourceShare(upId, value);
}

// ── Project Preferences Panel ──

async function togglePrefsPanel(up: UserProject) {
  const id = up.id;
  if (prefsOpen[id]) {
    prefsOpen[id] = false;
    return;
  }

  prefsOpen[id] = true;
  prefsLoading[id] = true;
  prefsError[id] = "";

  try {
    const result = await apiGetProjectPrefs(id);
    prefsData[id] = result.project_prefs || "";
  } catch (e: unknown) {
    prefsError[id] =
      e instanceof Error ? e.message : "Failed to load project preferences";
  } finally {
    prefsLoading[id] = false;
  }
}

async function saveProjectPrefs(upId: number) {
  prefsSaving[upId] = true;
  prefsError[upId] = "";
  prefsSuccess[upId] = "";

  try {
    await apiSetProjectPrefs(upId, prefsData[upId]);
    prefsSuccess[upId] = "Project preferences saved.";
  } catch (e: unknown) {
    prefsError[upId] =
      e instanceof Error ? e.message : "Failed to save project preferences";
  } finally {
    prefsSaving[upId] = false;
  }
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1 class="page-title">My Projects</h1>
    </div>

    <p v-if="store.userProjectsLoading" class="muted">Loading...</p>
    <p v-else-if="store.userProjectsError" class="error-banner">
      {{ store.userProjectsError }}
    </p>

    <div v-else-if="store.userProjects.length === 0" class="empty-state">
      <p class="empty-message">
        You haven't joined any projects yet.
        <RouterLink to="/projects">Browse the catalog</RouterLink>
      </p>
    </div>

    <div v-else class="my-projects-list">
      <div
        v-for="up in store.userProjects"
        :key="up.id"
        class="card my-project-card"
        :class="{
          suspended: up.suspended,
          detaching: up.pending_detach,
        }"
      >
        <!-- Header -->
        <div class="my-project-header">
          <h3>
            {{ up.project_name }}
            <span v-if="up.pending_detach" class="badge badge-warning">
              Detaching...
            </span>
            <span v-else-if="up.suspended" class="badge badge-default">
              Suspended
            </span>
            <span
              v-if="up.last_error"
              class="badge badge-danger"
              :title="up.last_error"
            >
              Error ({{ up.consecutive_failures }})
            </span>
          </h3>
          <a
            :href="up.project_url"
            target="_blank"
            rel="noopener"
            class="btn-link text-sm"
          >
            Visit
          </a>
        </div>

        <!-- Controls -->
        <div v-if="!up.pending_detach" class="my-project-controls">
          <label class="range-label">
            Resource share: <strong>{{ up.resource_share }}</strong>
            <input
              type="range"
              min="0"
              max="1000"
              step="10"
              :value="up.resource_share"
              :disabled="savingId === up.id"
              @change="onShareChange(up.id, $event)"
            />
          </label>

          <div class="checkbox-group">
            <label class="toggle-label">
              <input
                type="checkbox"
                :checked="up.suspended"
                :disabled="savingId === up.id"
                @change="toggleSuspend(up)"
              />
              Suspended
            </label>

            <label class="toggle-label">
              <input
                type="checkbox"
                :checked="up.dont_request_more_work"
                :disabled="savingId === up.id"
                @change="toggleDontRequestMoreWork(up)"
              />
              Don't request more work
            </label>
          </div>
        </div>

        <!-- No-RSC checkboxes -->
        <div v-if="!up.pending_detach" class="no-rsc-section">
          <span class="no-rsc-label text-sm muted">Disabled resources:</span>
          <div class="no-rsc-checks">
            <label
              v-for="rsc in RESOURCE_TYPES"
              :key="rsc.value"
              class="toggle-label"
            >
              <input
                type="checkbox"
                :checked="(up.no_rsc || []).includes(rsc.value)"
                :disabled="savingId === up.id"
                @change="toggleNoRsc(up, rsc.value)"
              />
              {{ rsc.label }}
            </label>
          </div>
        </div>

        <!-- Action buttons -->
        <div class="my-project-actions">
          <button
            v-if="!up.pending_detach"
            class="btn-secondary"
            @click="togglePrefsPanel(up)"
          >
            {{ prefsOpen[up.id] ? "Close Preferences" : "Project Preferences" }}
          </button>
          <button
            v-if="!up.pending_detach"
            class="btn-secondary"
            :disabled="detachingId === up.id"
            @click="detach(up.id)"
          >
            {{ detachingId === up.id ? "Detaching..." : "Detach" }}
          </button>
          <button
            class="btn-danger"
            :disabled="leavingId === up.id"
            @click="leave(up.id)"
          >
            {{ leavingId === up.id ? "Leaving..." : "Leave Project" }}
          </button>
        </div>

        <!-- Project Preferences Panel -->
        <div v-if="prefsOpen[up.id]" class="project-prefs-panel">
          <p v-if="prefsLoading[up.id]" class="muted text-sm">
            Loading project preferences...
          </p>
          <template v-else>
            <p v-if="prefsError[up.id]" class="error-banner">
              {{ prefsError[up.id] }}
            </p>
            <p v-if="prefsSuccess[up.id]" class="success-banner">
              {{ prefsSuccess[up.id] }}
            </p>
            <label class="prefs-xml-label">
              Project Preferences XML
              <textarea
                v-model="prefsData[up.id]"
                class="prefs-xml-textarea"
                rows="10"
                spellcheck="false"
              ></textarea>
            </label>
            <div class="prefs-xml-actions">
              <button
                class="btn-primary"
                :disabled="prefsSaving[up.id]"
                @click="saveProjectPrefs(up.id)"
              >
                {{ prefsSaving[up.id] ? "Saving..." : "Save" }}
              </button>
            </div>
          </template>
        </div>

        <!-- Phase 2 disabled section -->
        <div v-if="!up.pending_detach" class="phase2-section">
          <div class="phase2-header">
            <span class="text-sm muted">Per-task controls</span>
            <span class="phase2-badge">Requires Fresco</span>
          </div>
          <div class="phase2-controls">
            <label class="field-label">
              GPU usage per task
              <input type="number" disabled value="1.0" step="0.1" />
            </label>
            <label class="field-label">
              Max concurrent tasks
              <input type="number" disabled placeholder="Auto" />
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.checkbox-group {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-md);
}

.no-rsc-section {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.no-rsc-label {
  margin-right: var(--space-xs);
}

.no-rsc-checks {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-md);
}

/* Project prefs panel */
.project-prefs-panel {
  margin-top: var(--space-sm);
  padding-top: var(--space-md);
  border-top: 1px solid var(--color-border-light);
}

.prefs-xml-label {
  margin-bottom: 0;
}

.prefs-xml-textarea {
  margin-top: var(--space-sm);
  font-family: "Cascadia Code", "Fira Code", Consolas, monospace;
  font-size: var(--font-size-sm);
  line-height: 1.5;
  resize: vertical;
  min-height: 120px;
}

.prefs-xml-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: var(--space-sm);
}

/* Phase 2 disabled */
.phase2-section {
  margin-top: var(--space-sm);
  padding-top: var(--space-md);
  border-top: 1px dashed var(--color-border-light);
  opacity: 0.45;
  pointer-events: none;
}

.phase2-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
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

.phase2-controls {
  display: flex;
  gap: var(--space-lg);
  flex-wrap: wrap;
}

.field-label {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  font-weight: 500;
  margin-bottom: 0;
  min-width: 160px;
}

.field-label input {
  max-width: 160px;
}
</style>
