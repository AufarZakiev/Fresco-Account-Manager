<script setup lang="ts">
import { onMounted } from "vue";
import { useAuthStore } from "../stores/auth";
import { useProjectsStore } from "../stores/projects";

const auth = useAuthStore();
const projects = useProjectsStore();

onMounted(async () => {
  await projects.fetchUserProjects();
});
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1 class="page-title">Dashboard</h1>
    </div>

    <p class="greeting muted">
      Welcome back, <strong>{{ auth.user?.name ?? "User" }}</strong>
    </p>

    <div class="dashboard-cards">
      <div class="card">
        <h2>Enrolled Projects</h2>
        <p v-if="projects.userProjectsLoading" class="muted">Loading...</p>
        <template v-else>
          <p class="dashboard-card-value">{{ projects.userProjects.length }}</p>
          <RouterLink to="/my-projects" class="btn-link">
            Manage projects
          </RouterLink>
        </template>
      </div>

      <div class="card">
        <h2>Browse Projects</h2>
        <p class="muted text-sm">
          Find new BOINC projects to contribute to.
        </p>
        <RouterLink to="/projects" class="btn-link" style="margin-top: 8px; display: inline-block;">
          Project catalog
        </RouterLink>
      </div>

      <div class="card">
        <h2>Account</h2>
        <p class="muted text-sm">{{ auth.user?.email }}</p>
        <RouterLink to="/settings" class="btn-link" style="margin-top: 8px; display: inline-block;">
          Settings
        </RouterLink>
      </div>
    </div>
  </div>
</template>

<style scoped>
.greeting {
  margin-bottom: var(--space-lg);
  font-size: var(--font-size-base);
}

.greeting strong {
  color: var(--color-text-primary);
}
</style>
