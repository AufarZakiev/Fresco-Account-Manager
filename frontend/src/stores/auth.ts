import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  apiLogin,
  apiLogout,
  apiRegister,
  apiMe,
  ApiError,
  type User,
} from "../api/client";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<User | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const isAuthenticated = computed(() => user.value !== null);

  function clearError() {
    error.value = null;
  }

  function clear() {
    user.value = null;
    error.value = null;
    loading.value = false;
  }

  async function fetchMe(): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      user.value = await apiMe();
      return true;
    } catch (e) {
      user.value = null;
      if (e instanceof ApiError && e.status === 401) {
        // Not logged in -- that's fine
        return false;
      }
      error.value = e instanceof Error ? e.message : "Failed to fetch user";
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function login(email: string, password: string): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      user.value = await apiLogin(email, password);
      return true;
    } catch (e) {
      user.value = null;
      error.value = e instanceof Error ? e.message : "Login failed";
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function register(
    email: string,
    password: string,
    name: string,
  ): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      user.value = await apiRegister(email, password, name);
      return true;
    } catch (e) {
      user.value = null;
      error.value = e instanceof Error ? e.message : "Registration failed";
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function logout(): Promise<void> {
    try {
      await apiLogout();
    } catch {
      // Ignore errors on logout
    } finally {
      clear();
    }
  }

  return {
    user,
    loading,
    error,
    isAuthenticated,
    clearError,
    clear,
    fetchMe,
    login,
    register,
    logout,
  };
});
