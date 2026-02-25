<script setup lang="ts">
import { onMounted, ref, reactive } from "vue";
import {
  apiListHosts,
  apiGetHost,
  apiUpdateHost,
  type Host,
  type HostDetail,
} from "../api/client";

const hosts = ref<Host[]>([]);
const loading = ref(true);
const error = ref("");

const expandedId = ref<number | null>(null);
const hostDetails = reactive<Record<number, HostDetail>>({});
const detailLoading = ref<number | null>(null);
const savingVenueId = ref<number | null>(null);

onMounted(async () => {
  try {
    hosts.value = await apiListHosts();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : "Failed to load hosts";
  } finally {
    loading.value = false;
  }
});

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) return "just now";
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin} minute${diffMin === 1 ? "" : "s"} ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr} hour${diffHr === 1 ? "" : "s"} ago`;
  const diffDay = Math.floor(diffHr / 24);
  if (diffDay < 30) return `${diffDay} day${diffDay === 1 ? "" : "s"} ago`;

  return date.toLocaleDateString();
}

async function toggleExpand(host: Host) {
  if (expandedId.value === host.id) {
    expandedId.value = null;
    return;
  }

  expandedId.value = host.id;

  if (!hostDetails[host.id]) {
    detailLoading.value = host.id;
    try {
      hostDetails[host.id] = await apiGetHost(host.id);
    } catch (e: unknown) {
      error.value =
        e instanceof Error ? e.message : "Failed to load host details";
      expandedId.value = null;
    } finally {
      detailLoading.value = null;
    }
  }
}

async function onVenueChange(host: Host, event: Event) {
  const target = event.target as HTMLSelectElement;
  const newVenue = target.value;
  savingVenueId.value = host.id;
  try {
    await apiUpdateHost(host.id, newVenue);
    host.venue = newVenue;
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : "Failed to update host venue";
    target.value = host.venue;
  } finally {
    savingVenueId.value = null;
  }
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1 class="page-title">Hosts</h1>
    </div>

    <p v-if="loading" class="muted">Loading...</p>
    <p v-else-if="error" class="error-banner">{{ error }}</p>

    <div v-else-if="hosts.length === 0" class="empty-state">
      <p class="empty-message">No hosts have connected yet.</p>
    </div>

    <div v-else class="hosts-list">
      <div
        v-for="host in hosts"
        :key="host.id"
        class="card host-card"
        @click="toggleExpand(host)"
      >
        <div class="host-header">
          <h3>{{ host.domain_name || `Host #${host.id}` }}</h3>
          <span class="text-xs muted">
            {{ expandedId === host.id ? "Collapse" : "Details" }}
          </span>
        </div>

        <div class="host-info">
          <div class="host-field">
            <span class="host-label">Platform</span>
            <span>{{ host.platform_name || "Unknown" }}</span>
          </div>
          <div class="host-field">
            <span class="host-label">Client</span>
            <span>{{ host.client_version }}</span>
          </div>
          <div class="host-field">
            <span class="host-label">Run mode</span>
            <span>{{ host.run_mode || "auto" }}</span>
          </div>
          <div class="host-field">
            <span class="host-label">Last contact</span>
            <span :title="host.last_rpc_at">{{
              formatRelativeTime(host.last_rpc_at)
            }}</span>
          </div>
          <div class="host-field" @click.stop>
            <span class="host-label">Venue</span>
            <select
              :value="host.venue"
              :disabled="savingVenueId === host.id"
              @change="onVenueChange(host, $event)"
            >
              <option value="">(default)</option>
              <option value="home">home</option>
              <option value="work">work</option>
              <option value="school">school</option>
            </select>
          </div>
        </div>

        <div
          v-if="expandedId === host.id"
          class="host-detail"
          @click.stop
        >
          <h3>Host Info XML</h3>
          <p v-if="detailLoading === host.id" class="muted">Loading...</p>
          <pre
            v-else-if="hostDetails[host.id]"
            class="host-xml"
          >{{ hostDetails[host.id]?.host_info_xml }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>
