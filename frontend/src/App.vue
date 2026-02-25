<script setup lang="ts">
import { computed } from "vue";
import { useAuthStore } from "./stores/auth";
import AppSidebar from "./components/AppSidebar.vue";

const auth = useAuthStore();
const hasSidebar = computed(() => auth.isAuthenticated);
</script>

<template>
  <div class="app" :class="{ 'has-sidebar': hasSidebar }">
    <AppSidebar />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>
