<script setup lang="ts">
import { onMounted, ref } from "vue";
import { apiGetUserStats, type UserStats } from "../api/client";

const stats = ref<UserStats | null>(null);
const loading = ref(true);
const error = ref("");

function fmtCredit(n: number): string {
  return n.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

onMounted(async () => {
  try {
    stats.value = await apiGetUserStats();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : "Failed to load stats";
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="page">
    <h1>Statistics</h1>

    <p v-if="loading" class="muted">Loading...</p>
    <p v-else-if="error" class="error-banner">{{ error }}</p>

    <template v-else-if="stats">
      <div class="stats-cards">
        <div class="card stat-card">
          <span class="stat-label">Total Credit</span>
          <span class="stat-value">{{ fmtCredit(stats.total_credit) }}</span>
        </div>
        <div class="card stat-card">
          <span class="stat-label">Recent Credit</span>
          <span class="stat-value">{{ fmtCredit(stats.recent_credit) }}</span>
        </div>
        <div class="card stat-card">
          <span class="stat-label">Projects</span>
          <span class="stat-value">{{ stats.project_count }}</span>
        </div>
        <div class="card stat-card">
          <span class="stat-label">Hosts</span>
          <span class="stat-value">{{ stats.host_count }}</span>
        </div>
      </div>

      <div class="card projects-table-card">
        <h2>Per-Project Credits</h2>
        <p v-if="stats.projects.length === 0" class="muted">
          No project credit data available.
        </p>
        <table v-else class="credits-table">
          <thead>
            <tr>
              <th>Project</th>
              <th class="num">Total Credit</th>
              <th class="num">Recent Credit</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="p in stats.projects" :key="p.project_name">
              <td>{{ p.project_name }}</td>
              <td class="num">{{ fmtCredit(p.total_credit) }}</td>
              <td class="num">{{ fmtCredit(p.recent_credit) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>

<style scoped>
.stats-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 1.25rem;
}

.stat-label {
  font-size: 0.85rem;
  color: var(--c-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: 0.5rem;
}

.stat-value {
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--c-text-heading);
}

.projects-table-card {
  overflow-x: auto;
}

.credits-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 0.75rem;
}

.credits-table th,
.credits-table td {
  padding: 0.6rem 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--c-border);
}

.credits-table th {
  font-size: 0.8rem;
  color: var(--c-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.credits-table td {
  font-size: 0.95rem;
}

.credits-table .num {
  text-align: right;
  font-variant-numeric: tabular-nums;
}
</style>
