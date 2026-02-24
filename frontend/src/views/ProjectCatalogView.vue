<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useProjectsStore } from "../stores/projects";

const store = useProjectsStore();

const search = ref("");
const enrollingId = ref<number | null>(null);
const enrolledIds = ref<Set<number>>(new Set());

onMounted(async () => {
  await Promise.all([store.fetchCatalog(), store.fetchUserProjects()]);
  // Build a set of already-enrolled project IDs for quick lookup
  for (const up of store.userProjects) {
    enrolledIds.value.add(up.project_id);
  }
});

const filtered = computed(() => {
  const q = search.value.toLowerCase().trim();
  if (!q) return store.catalog;
  return store.catalog.filter(
    (p) =>
      p.name.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      p.general_area.toLowerCase().includes(q) ||
      p.specific_area.toLowerCase().includes(q),
  );
});

async function join(projectId: number) {
  enrollingId.value = projectId;
  const ok = await store.enroll(projectId);
  if (ok) {
    enrolledIds.value.add(projectId);
  }
  enrollingId.value = null;
}
</script>

<template>
  <div class="page">
    <h1>Project Catalog</h1>

    <input
      v-model="search"
      type="search"
      placeholder="Search projects..."
      class="search-input"
    />

    <p v-if="store.catalogLoading" class="muted">Loading projects...</p>
    <p v-else-if="store.catalogError" class="error-banner">
      {{ store.catalogError }}
    </p>
    <p v-else-if="filtered.length === 0" class="muted">No projects found.</p>

    <div v-else class="project-grid">
      <div
        v-for="project in filtered"
        :key="project.id"
        class="card project-card"
      >
        <h3>{{ project.name }}</h3>
        <p class="muted small">
          {{ project.general_area }}
          <template v-if="project.specific_area">
            / {{ project.specific_area }}
          </template>
        </p>
        <p class="project-desc">{{ project.description }}</p>
        <div class="project-actions">
          <a
            v-if="project.home_url"
            :href="project.home_url"
            target="_blank"
            rel="noopener"
            class="btn-link"
          >
            Website
          </a>
          <button
            v-if="enrolledIds.has(project.id)"
            disabled
            class="btn-secondary"
          >
            Joined
          </button>
          <button
            v-else
            :disabled="enrollingId === project.id"
            class="btn-primary"
            @click="join(project.id)"
          >
            {{ enrollingId === project.id ? "Joining..." : "Join" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
