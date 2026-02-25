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
    <div class="page-header">
      <h1 class="page-title">Statistics</h1>
    </div>

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

      <div class="data-table-wrapper">
        <h2 style="padding: 12px 12px 0;">Per-Project Credits</h2>
        <div v-if="stats.projects.length === 0" class="empty-state">
          <p class="empty-message">No project credit data available.</p>
        </div>
        <table v-else class="data-table">
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
