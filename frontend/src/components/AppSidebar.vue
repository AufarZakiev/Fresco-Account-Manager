<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";

const auth = useAuthStore();
const router = useRouter();
const mobileOpen = ref(false);

async function handleLogout() {
  await auth.logout();
  router.push("/login");
}

function closeMobile() {
  mobileOpen.value = false;
}

function onNavClick() {
  mobileOpen.value = false;
}
</script>

<template>
  <!-- Mobile hamburger -->
  <button
    v-if="auth.isAuthenticated"
    class="hamburger-btn"
    @click="mobileOpen = !mobileOpen"
  >
    <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
      <path
        d="M2 4h14M2 9h14M2 14h14"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
      />
    </svg>
  </button>

  <!-- Mobile backdrop -->
  <div
    v-if="mobileOpen"
    class="sidebar-backdrop"
    @click="closeMobile"
  ></div>

  <!-- Sidebar -->
  <aside
    v-if="auth.isAuthenticated"
    class="sidebar"
    :class="{ open: mobileOpen }"
  >
    <div class="sidebar-brand">
      <RouterLink to="/" class="brand-link" @click="onNavClick">
        FAM
      </RouterLink>
    </div>

    <nav class="sidebar-nav">
      <div class="nav-group">
        <span class="nav-group-label">Navigation</span>
        <RouterLink to="/" class="nav-item" @click="onNavClick">
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1L1 7h2v6h4V9h2v4h4V7h2L8 1z" />
          </svg>
          Dashboard
        </RouterLink>
        <RouterLink to="/projects" class="nav-item" @click="onNavClick">
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2 3h12v2H2V3zm0 4h12v2H2V7zm0 4h8v2H2v-2z" />
          </svg>
          Projects
        </RouterLink>
        <RouterLink to="/my-projects" class="nav-item" @click="onNavClick">
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 0L6 6H0l5 3.5L3 16l5-3.5L13 16l-2-6.5L16 6h-6L8 0z" />
          </svg>
          My Projects
        </RouterLink>
        <RouterLink to="/hosts" class="nav-item" @click="onNavClick">
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M2 2h12v4H2V2zm0 6h12v4H2V8zm2-4h2v1H4V4zm0 6h2v1H4v-1z" />
          </svg>
          Hosts
        </RouterLink>
        <RouterLink to="/stats" class="nav-item" @click="onNavClick">
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M1 14h14v1H1v-1zm1-4h2v4H2v-4zm4-3h2v7H6V7zm4-4h2v11h-2V3zm-8 8h2v3H2v-3z" />
          </svg>
          Stats
        </RouterLink>
      </div>

      <div class="nav-group">
        <span class="nav-group-label">Settings</span>
        <RouterLink to="/preferences" class="nav-item" @click="onNavClick">
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M6.5 1L6 3.1a5.5 5.5 0 0 0-1.4.8L2.5 3 1 5.6l1.8 1.5a5.5 5.5 0 0 0 0 1.6L1 10.4 2.5 13l2.1-.9c.4.3.9.6 1.4.8L6.5 15h3l.5-2.1c.5-.2 1-.5 1.4-.8l2.1.9L15 10.4l-1.8-1.5a5.5 5.5 0 0 0 0-1.6L15 5.6 13.5 3l-2.1.9A5.5 5.5 0 0 0 10 3.1L9.5 1h-3zM8 5.5a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5z" />
          </svg>
          Preferences
        </RouterLink>
        <RouterLink to="/settings" class="nav-item" @click="onNavClick">
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3zm-5 11c0-2 2-3.5 5-3.5s5 1.5 5 3.5v1H3v-1z" />
          </svg>
          Account
        </RouterLink>
        <RouterLink
          v-if="auth.user?.is_admin"
          to="/admin"
          class="nav-item"
          @click="onNavClick"
        >
          <svg class="nav-icon" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1L2 4v4c0 3.5 2.5 6.5 6 7.5 3.5-1 6-4 6-7.5V4L8 1zm0 2.2L12 5v3c0 2.8-1.8 5.1-4 6-2.2-.9-4-3.2-4-6V5l4-1.8z" />
          </svg>
          Admin
        </RouterLink>
      </div>
    </nav>

    <div class="sidebar-footer">
      <div class="sidebar-user">
        <span class="sidebar-user-name">{{ auth.user?.name }}</span>
        <span class="sidebar-user-email">{{ auth.user?.email }}</span>
      </div>
      <div class="sidebar-actions">
        <button
          class="sidebar-action-btn"
          title="Sign out"
          @click="handleLogout"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M6 2H3a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h3v-1.5H3.5v-9H6V2zm4.5 2.5L9 6l2 2H5v1h6l-2 2 1.5 1.5L15 8l-4.5-3.5z" />
          </svg>
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  width: var(--sidebar-width);
  background: var(--color-bg-secondary);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  z-index: var(--z-sidebar-overlay);
  overflow-y: auto;
}

.sidebar-brand {
  padding: var(--space-lg) var(--space-lg) var(--space-md);
}

.brand-link {
  font-weight: 700;
  font-size: var(--font-size-lg);
  color: var(--color-text-primary);
  letter-spacing: 0.05em;
  text-decoration: none;
}

.brand-link:hover {
  text-decoration: none;
  color: var(--color-accent);
}

.sidebar-nav {
  flex: 1;
  padding: var(--space-xs) var(--space-sm);
}

.nav-group {
  margin-bottom: var(--space-lg);
}

.nav-group-label {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: var(--space-xs) 10px;
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: 7px 10px;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-md);
  text-decoration: none;
  color: var(--color-text-secondary);
  transition: all var(--transition-fast);
  font-weight: 450;
}

.nav-item:hover {
  background: var(--color-bg-tertiary);
  color: var(--color-text-primary);
  text-decoration: none;
}

.nav-item.router-link-exact-active {
  background: var(--color-accent-light);
  color: var(--color-accent);
  font-weight: 550;
}

.nav-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  opacity: 0.7;
}

.nav-item.router-link-exact-active .nav-icon {
  opacity: 1;
}

.sidebar-footer {
  padding: var(--space-md);
  border-top: 1px solid var(--color-border);
}

.sidebar-user {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: var(--space-sm);
}

.sidebar-user-name {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-primary);
}

.sidebar-user-email {
  font-size: var(--font-size-xs);
  color: var(--color-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-actions {
  display: flex;
  justify-content: flex-end;
}

.sidebar-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.sidebar-action-btn:hover {
  background: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

/* Mobile hamburger */
.hamburger-btn {
  display: none;
  position: fixed;
  top: 8px;
  left: 8px;
  z-index: var(--z-nav-header);
  width: 36px;
  height: 36px;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg);
  color: var(--color-text-primary);
  cursor: pointer;
}

.sidebar-backdrop {
  display: none;
}

@media (max-width: 767px) {
  .hamburger-btn {
    display: flex;
  }

  .sidebar-backdrop {
    display: block;
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: var(--z-content);
  }

  .sidebar {
    transform: translateX(-100%);
    transition: transform var(--transition-normal);
  }

  .sidebar.open {
    transform: translateX(0);
  }
}
</style>
