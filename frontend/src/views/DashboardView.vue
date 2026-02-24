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
    <h1>Dashboard</h1>

    <p class="greeting">
      Welcome back, <strong>{{ auth.user?.name ?? "User" }}</strong>
    </p>

    <div class="dashboard-cards">
      <div class="card">
        <h2>Enrolled Projects</h2>
        <p v-if="projects.userProjectsLoading" class="muted">Loading...</p>
        <template v-else>
          <p class="big-number">{{ projects.userProjects.length }}</p>
          <RouterLink to="/my-projects" class="card-link">
            Manage projects
          </RouterLink>
        </template>
      </div>

      <div class="card">
        <h2>Browse Projects</h2>
        <p class="muted">
          Find new BOINC projects to contribute to.
        </p>
        <RouterLink to="/projects" class="card-link">
          Project catalog
        </RouterLink>
      </div>

      <div class="card">
        <h2>Account</h2>
        <p class="muted">{{ auth.user?.email }}</p>
        <RouterLink to="/settings" class="card-link">
          Settings
        </RouterLink>
      </div>
    </div>
  </div>
</template>
