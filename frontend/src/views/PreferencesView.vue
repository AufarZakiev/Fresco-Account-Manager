<script setup lang="ts">
import { onMounted, ref } from "vue";
import { apiGetPreferences, apiSetPreferences } from "../api/client";

const prefsXml = ref("");
const modTime = ref("");
const loading = ref(true);
const saving = ref(false);
const error = ref("");
const success = ref("");

onMounted(async () => {
  try {
    const prefs = await apiGetPreferences();
    prefsXml.value = prefs.prefs_xml;
    modTime.value = prefs.mod_time;
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : "Failed to load preferences";
  } finally {
    loading.value = false;
  }
});

function formatDate(dateStr: string): string {
  if (!dateStr) return "";
  const d = new Date(dateStr);
  return d.toLocaleString();
}

async function handleSave() {
  saving.value = true;
  error.value = "";
  success.value = "";

  try {
    await apiSetPreferences(prefsXml.value);
    success.value = "Preferences saved successfully.";
    // Update mod_time to now
    modTime.value = new Date().toISOString();
  } catch (e: unknown) {
    error.value =
      e instanceof Error ? e.message : "Failed to save preferences";
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="page">
    <h1>Global Preferences</h1>

    <p v-if="loading" class="muted">Loading...</p>

    <template v-else>
      <p v-if="error" class="error-banner">{{ error }}</p>
      <p v-if="success" class="success-banner">{{ success }}</p>

      <div class="card prefs-card">
        <div class="prefs-meta">
          <span class="muted small">
            Last modified: {{ modTime ? formatDate(modTime) : "never" }}
          </span>
        </div>

        <label class="prefs-label">
          Preferences XML
          <textarea
            v-model="prefsXml"
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
.prefs-card {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.prefs-meta {
  display: flex;
  justify-content: flex-end;
}

.prefs-label {
  margin-bottom: 0;
}

.prefs-textarea {
  margin-top: 0.5rem;
  font-family: "Cascadia Code", "Fira Code", "Consolas", monospace;
  font-size: 0.85rem;
  line-height: 1.5;
  resize: vertical;
  min-height: 200px;
}

.prefs-actions {
  display: flex;
  justify-content: flex-end;
}

.success-banner {
  background-color: rgba(46, 204, 113, 0.15);
  border: 1px solid #2ecc71;
  border-radius: var(--radius);
  color: #2ecc71;
  padding: 0.5rem 0.75rem;
  font-size: 0.9rem;
  margin-bottom: 1rem;
}
</style>
