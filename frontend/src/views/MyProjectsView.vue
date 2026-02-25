<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useProjectsStore } from "../stores/projects";
import {
  apiSuspendProject,
  apiResumeProject,
  apiDetachProject,
} from "../api/client";

const store = useProjectsStore();

const savingId = ref<number | null>(null);
const leavingId = ref<number | null>(null);
const detachingId = ref<number | null>(null);

onMounted(async () => {
  await store.fetchUserProjects();
});

async function toggleSuspend(up: { id: number; suspended: boolean }) {
  savingId.value = up.id;
  if (up.suspended) {
    await apiResumeProject(up.id);
  } else {
    await apiSuspendProject(up.id);
  }
  await store.fetchUserProjects();
  savingId.value = null;
}

async function saveResourceShare(upId: number, value: number) {
  savingId.value = upId;
  await store.updateProject(upId, { resource_share: value });
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

          <label class="toggle-label">
            <input
              type="checkbox"
              :checked="up.suspended"
              :disabled="savingId === up.id"
              @change="toggleSuspend(up)"
            />
            Suspended
          </label>
        </div>

        <div class="my-project-actions">
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
      </div>
    </div>
  </div>
</template>
