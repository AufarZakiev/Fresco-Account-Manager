import {
  createRouter,
  createWebHistory,
  type RouteLocationNormalized,
} from "vue-router";
import { useAuthStore } from "../stores/auth";

import LoginView from "../views/LoginView.vue";
import RegisterView from "../views/RegisterView.vue";
import DashboardView from "../views/DashboardView.vue";
import ProjectCatalogView from "../views/ProjectCatalogView.vue";
import MyProjectsView from "../views/MyProjectsView.vue";
import SettingsView from "../views/SettingsView.vue";
import HostsView from "../views/HostsView.vue";
import PreferencesView from "../views/PreferencesView.vue";
import StatsView from "../views/StatsView.vue";
import AdminView from "../views/AdminView.vue";

const routes = [
  {
    path: "/login",
    name: "login",
    component: LoginView,
    meta: { guest: true },
  },
  {
    path: "/register",
    name: "register",
    component: RegisterView,
    meta: { guest: true },
  },
  {
    path: "/",
    name: "dashboard",
    component: DashboardView,
    meta: { requiresAuth: true },
  },
  {
    path: "/projects",
    name: "project-catalog",
    component: ProjectCatalogView,
    meta: { requiresAuth: true },
  },
  {
    path: "/my-projects",
    name: "my-projects",
    component: MyProjectsView,
    meta: { requiresAuth: true },
  },
  {
    path: "/settings",
    name: "settings",
    component: SettingsView,
    meta: { requiresAuth: true },
  },
  {
    path: "/hosts",
    name: "hosts",
    component: HostsView,
    meta: { requiresAuth: true },
  },
  {
    path: "/preferences",
    name: "preferences",
    component: PreferencesView,
    meta: { requiresAuth: true },
  },
  {
    path: "/stats",
    name: "stats",
    component: StatsView,
    meta: { requiresAuth: true },
  },
  {
    path: "/admin",
    name: "admin",
    component: AdminView,
    meta: { requiresAuth: true },
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

let authInitialized = false;

router.beforeEach(
  async (to: RouteLocationNormalized, _from: RouteLocationNormalized) => {
    const auth = useAuthStore();

    // On first navigation, try to restore the session from the server cookie
    if (!authInitialized) {
      authInitialized = true;
      await auth.fetchMe();
    }

    if (to.meta.requiresAuth && !auth.isAuthenticated) {
      return { name: "login", query: { redirect: to.fullPath } };
    }

    if (to.meta.guest && auth.isAuthenticated) {
      return { name: "dashboard" };
    }
  },
);

export default router;
