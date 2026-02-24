<script setup lang="ts">
import { useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";

const auth = useAuthStore();
const router = useRouter();

async function handleLogout() {
  await auth.logout();
  router.push("/login");
}
</script>

<template>
  <header class="app-header">
    <nav class="app-nav">
      <RouterLink to="/" class="nav-brand">FAM</RouterLink>

      <template v-if="auth.isAuthenticated">
        <div class="nav-links">
          <RouterLink to="/">Dashboard</RouterLink>
          <RouterLink to="/projects">Projects</RouterLink>
          <RouterLink to="/my-projects">My Projects</RouterLink>
          <RouterLink to="/hosts">Hosts</RouterLink>
          <RouterLink to="/preferences">Preferences</RouterLink>
          <RouterLink to="/stats">Stats</RouterLink>
          <RouterLink to="/settings">Settings</RouterLink>
          <RouterLink v-if="auth.user?.is_admin" to="/admin">Admin</RouterLink>
        </div>
        <div class="nav-right">
          <span class="nav-user">{{ auth.user?.name }}</span>
          <button class="btn-link" @click="handleLogout">Sign Out</button>
        </div>
      </template>

      <template v-else>
        <div class="nav-links"></div>
        <div class="nav-right">
          <RouterLink to="/login">Sign In</RouterLink>
          <RouterLink to="/register">Register</RouterLink>
        </div>
      </template>
    </nav>
  </header>
</template>
