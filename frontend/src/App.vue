<script setup lang="ts">
import { computed } from "vue";
import { useAuthStore } from "./stores/auth";
import AppSidebar from "./components/AppSidebar.vue";
import { useTheme } from "./composables/useTheme";

const auth = useAuthStore();
const hasSidebar = computed(() => auth.isAuthenticated);
const { theme, cycle: cycleTheme } = useTheme();
</script>

<template>
  <div class="app" :class="{ 'has-sidebar': hasSidebar }">
    <AppSidebar />

    <!-- Theme toggle for unauthenticated pages (login/register) -->
    <button
      v-if="!hasSidebar"
      class="theme-toggle-corner"
      :title="`Theme: ${theme}`"
      @click="cycleTheme"
    >
      <svg v-if="theme === 'light'" width="16" height="16" viewBox="0 0 16 16">
        <path d="M8 1v2m0 10v2M1 8h2m10 0h2M3.05 3.05l1.41 1.41m7.08 7.08l1.41 1.41M3.05 12.95l1.41-1.41m7.08-7.08l1.41-1.41M8 4.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" fill="none" />
      </svg>
      <svg v-else-if="theme === 'dark'" width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M6 2a6 6 0 1 0 8 8c-3.3 0-6-2.7-6-6a6 6 0 0 0-2-2z" />
      </svg>
      <svg v-else width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 1.5v11a5.5 5.5 0 0 1 0-11z" />
      </svg>
    </button>

    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.theme-toggle-corner {
  position: fixed;
  top: 12px;
  right: 12px;
  z-index: var(--z-nav-header);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.theme-toggle-corner:hover {
  background: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}
</style>
