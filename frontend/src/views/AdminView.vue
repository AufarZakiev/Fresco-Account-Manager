<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAuthStore } from "../stores/auth";
import {
  apiAdminGetStats,
  apiAdminListUsers,
  apiAdminCreateProject,
  apiAdminImportBoinc,
  ApiError,
  type AdminStats,
  type AdminUser,
} from "../api/client";

const auth = useAuthStore();

const stats = ref<AdminStats | null>(null);
const users = ref<AdminUser[]>([]);
const loading = ref(true);
const error = ref("");

// New project form
const projectUrl = ref("");
const projectName = ref("");
const projectDesc = ref("");
const projectSubmitting = ref(false);
const projectError = ref<string | null>(null);
const projectSuccess = ref<string | null>(null);

// Import from BOINC
const importLoading = ref(false);
const importResult = ref<string | null>(null);
const importError = ref<string | null>(null);

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString();
}

onMounted(async () => {
  if (!auth.user?.is_admin) {
    loading.value = false;
    return;
  }

  try {
    const [s, u] = await Promise.all([
      apiAdminGetStats(),
      apiAdminListUsers(),
    ]);
    stats.value = s;
    users.value = u;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : "Failed to load admin data";
  } finally {
    loading.value = false;
  }
});

async function onImportBoinc() {
  importResult.value = null;
  importError.value = null;
  importLoading.value = true;
  try {
    const res = await apiAdminImportBoinc();
    importResult.value = `Fetched ${res.total_fetched} projects: ${res.imported} imported, ${res.skipped} skipped.`;
    stats.value = await apiAdminGetStats();
  } catch (e: unknown) {
    if (e instanceof ApiError) {
      importError.value = e.message || "Failed to import projects.";
    } else {
      importError.value = "Failed to import projects.";
    }
  } finally {
    importLoading.value = false;
  }
}

async function onCreateProject() {
  projectError.value = null;
  projectSuccess.value = null;

  if (!projectUrl.value || !projectName.value) {
    projectError.value = "URL and Name are required.";
    return;
  }

  projectSubmitting.value = true;
  try {
    await apiAdminCreateProject({
      url: projectUrl.value,
      name: projectName.value,
      description: projectDesc.value || undefined,
    });
    projectSuccess.value = `Project "${projectName.value}" created successfully.`;
    projectUrl.value = "";
    projectName.value = "";
    projectDesc.value = "";
  } catch (e: unknown) {
    if (e instanceof ApiError) {
      projectError.value = e.message || "Failed to create project.";
    } else {
      projectError.value = "Failed to create project.";
    }
  } finally {
    projectSubmitting.value = false;
  }
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1 class="page-title">Admin Panel</h1>
    </div>

    <div v-if="!auth.user?.is_admin" class="card access-denied">
      <h2>Access Denied</h2>
      <p class="muted">You do not have administrator privileges to view this page.</p>
    </div>

    <template v-else>
      <p v-if="loading" class="muted">Loading...</p>
      <p v-else-if="error" class="error-banner">{{ error }}</p>

      <template v-else>
        <!-- System Stats -->
        <div class="stats-cards">
          <div v-if="stats" class="card stat-card">
            <span class="stat-label">Users</span>
            <span class="stat-value">{{ stats.total_users }}</span>
          </div>
          <div v-if="stats" class="card stat-card">
            <span class="stat-label">Hosts</span>
            <span class="stat-value">{{ stats.total_hosts }}</span>
          </div>
          <div v-if="stats" class="card stat-card">
            <span class="stat-label">Projects</span>
            <span class="stat-value">{{ stats.total_projects }}</span>
          </div>
          <div v-if="stats" class="card stat-card">
            <span class="stat-label">Enrollments</span>
            <span class="stat-value">{{ stats.total_enrollments }}</span>
          </div>
          <div v-if="stats" class="card stat-card">
            <span class="stat-label">Active Sessions</span>
            <span class="stat-value">{{ stats.active_sessions }}</span>
          </div>
        </div>

        <!-- User List -->
        <div class="card admin-section">
          <h2>Users</h2>
          <div class="data-table-wrapper">
            <table class="data-table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Email</th>
                  <th>Name</th>
                  <th>Admin</th>
                  <th>Created</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="u in users" :key="u.id">
                  <td>{{ u.id }}</td>
                  <td>{{ u.email }}</td>
                  <td>{{ u.name }}</td>
                  <td>
                    <span v-if="u.is_admin" class="badge badge-info">Yes</span>
                    <span v-else class="badge badge-default">No</span>
                  </td>
                  <td>{{ formatDate(u.created_at) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- Import from BOINC -->
        <div class="card admin-section">
          <h2>Import Projects from BOINC</h2>
          <p class="muted text-sm">
            Fetch the official BOINC project list and import all projects into the catalog.
            Existing projects are updated with the latest metadata.
          </p>
          <div v-if="importResult" class="success-banner">{{ importResult }}</div>
          <div v-if="importError" class="error-banner">{{ importError }}</div>
          <button
            class="btn-primary"
            :disabled="importLoading"
            @click="onImportBoinc"
          >
            {{ importLoading ? "Importing..." : "Import from BOINC" }}
          </button>
        </div>

        <!-- Add Project -->
        <div class="card admin-section">
          <h2>Add New Project</h2>

          <div v-if="projectSuccess" class="success-banner">{{ projectSuccess }}</div>
          <div v-if="projectError" class="error-banner">{{ projectError }}</div>

          <form class="add-project-form" @submit.prevent="onCreateProject">
            <label>
              Project URL
              <input
                v-model="projectUrl"
                type="text"
                required
                placeholder="https://project.example.com/"
              />
            </label>

            <label>
              Project Name
              <input
                v-model="projectName"
                type="text"
                required
                placeholder="Project name"
              />
            </label>

            <label>
              Description
              <textarea
                v-model="projectDesc"
                rows="3"
                placeholder="Optional description"
              ></textarea>
            </label>

            <button
              type="submit"
              class="btn-primary"
              :disabled="projectSubmitting"
            >
              {{ projectSubmitting ? "Creating..." : "Create Project" }}
            </button>
          </form>
        </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.access-denied {
  text-align: center;
  padding: var(--space-2xl) var(--space-lg);
}

.admin-section {
  margin-bottom: var(--space-lg);
}

.admin-section .data-table-wrapper {
  border: none;
  border-radius: 0;
  margin-top: var(--space-sm);
}

.add-project-form {
  max-width: 500px;
}

.add-project-form .btn-primary {
  margin-top: var(--space-sm);
}
</style>
