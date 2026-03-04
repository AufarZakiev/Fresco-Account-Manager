import { defineStore } from "pinia";
import { ref } from "vue";
import {
  apiListProjects,
  apiGetProject,
  apiListUserProjects,
  apiEnrollProject,
  apiUpdateUserProject,
  apiLeaveProject,
  type Project,
  type UserProject,
  type UserProjectPatch,
} from "../api/client";

export const useProjectsStore = defineStore("projects", () => {
  // -- Catalog --
  const catalog = ref<Project[]>([]);
  const catalogLoading = ref(false);
  const catalogError = ref<string | null>(null);

  async function fetchCatalog(): Promise<void> {
    catalogLoading.value = true;
    catalogError.value = null;
    try {
      catalog.value = await apiListProjects();
    } catch (e) {
      catalogError.value =
        e instanceof Error ? e.message : "Failed to load projects";
    } finally {
      catalogLoading.value = false;
    }
  }

  async function fetchProject(id: number): Promise<Project | null> {
    try {
      return await apiGetProject(id);
    } catch {
      return null;
    }
  }

  // -- User projects --
  const userProjects = ref<UserProject[]>([]);
  const userProjectsLoading = ref(false);
  const userProjectsError = ref<string | null>(null);

  async function fetchUserProjects(): Promise<void> {
    userProjectsLoading.value = true;
    userProjectsError.value = null;
    try {
      userProjects.value = await apiListUserProjects();
    } catch (e) {
      userProjectsError.value =
        e instanceof Error ? e.message : "Failed to load enrolled projects";
    } finally {
      userProjectsLoading.value = false;
    }
  }

  async function enroll(projectId: number): Promise<boolean> {
    try {
      await apiEnrollProject(projectId);
      await fetchUserProjects();
      return true;
    } catch {
      return false;
    }
  }

  async function updateProject(
    id: number,
    patch: UserProjectPatch,
  ): Promise<boolean> {
    try {
      await apiUpdateUserProject(id, patch);
      // Optimistic local update for known fields
      const idx = userProjects.value.findIndex((p) => p.id === id);
      if (idx !== -1) {
        const current = userProjects.value[idx];
        userProjects.value[idx] = { ...current, ...patch };
      }
      return true;
    } catch {
      return false;
    }
  }

  async function leave(id: number): Promise<boolean> {
    try {
      await apiLeaveProject(id);
      userProjects.value = userProjects.value.filter((p) => p.id !== id);
      return true;
    } catch {
      return false;
    }
  }

  return {
    catalog,
    catalogLoading,
    catalogError,
    fetchCatalog,
    fetchProject,
    userProjects,
    userProjectsLoading,
    userProjectsError,
    fetchUserProjects,
    enroll,
    updateProject,
    leave,
  };
});
