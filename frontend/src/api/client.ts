const BASE_URL = "/api";

export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export async function apiFetch<T>(
  path: string,
  options?: RequestInit,
): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
    credentials: "same-origin",
    ...options,
  });

  if (!response.ok) {
    const text = await response.text();
    throw new ApiError(response.status, text);
  }

  // Handle 204 No Content
  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

// ---------- Types ----------

export interface User {
  id: number;
  email: string;
  name: string;
  country: string;
  is_admin: boolean;
}

export interface Project {
  id: number;
  url: string;
  name: string;
  description: string;
  general_area: string;
  specific_area: string;
  home_url: string;
  is_active: boolean;
}

export interface UserProject {
  id: number;
  project_id: number;
  project_name: string;
  project_url: string;
  resource_share: number;
  suspended: boolean;
  dont_request_more_work: boolean;
  has_authenticator: boolean;
  pending_detach: boolean;
  detach_when_done: boolean;
  last_error: string | null;
  consecutive_failures: number;
}

export interface UserProjectPatch {
  resource_share?: number;
  suspended?: boolean;
  dont_request_more_work?: boolean;
}

// ---------- Auth API ----------

export function apiRegister(email: string, password: string, name: string) {
  return apiFetch<User>("/auth/register", {
    method: "POST",
    body: JSON.stringify({ email, password, name }),
  });
}

export function apiLogin(email: string, password: string) {
  return apiFetch<User>("/auth/login", {
    method: "POST",
    body: JSON.stringify({ email, password }),
  });
}

export function apiLogout() {
  return apiFetch<void>("/auth/logout", { method: "POST" });
}

export function apiMe() {
  return apiFetch<User>("/auth/me");
}

// ---------- Projects API ----------

export function apiListProjects() {
  return apiFetch<Project[]>("/projects");
}

export function apiGetProject(id: number) {
  return apiFetch<Project>(`/projects/${id}`);
}

// ---------- User Projects API ----------

export function apiListUserProjects() {
  return apiFetch<UserProject[]>("/user/projects");
}

export function apiEnrollProject(projectId: number) {
  return apiFetch<UserProject>("/user/projects", {
    method: "POST",
    body: JSON.stringify({ project_id: projectId }),
  });
}

export function apiUpdateUserProject(id: number, patch: UserProjectPatch) {
  return apiFetch<UserProject>(`/user/projects/${id}`, {
    method: "PATCH",
    body: JSON.stringify(patch),
  });
}

export function apiLeaveProject(id: number) {
  return apiFetch<void>(`/user/projects/${id}`, { method: "DELETE" });
}

export function apiSuspendProject(id: number) {
  return apiFetch<void>(`/user/projects/${id}/suspend`, { method: "POST" });
}

export function apiResumeProject(id: number) {
  return apiFetch<void>(`/user/projects/${id}/resume`, { method: "POST" });
}

export function apiDetachProject(id: number) {
  return apiFetch<void>(`/user/projects/${id}/detach`, { method: "POST" });
}

// ---------- Host Types ----------

export interface Host {
  id: number;
  host_cpid: string;
  domain_name: string;
  client_version: string;
  platform_name: string;
  venue: string;
  run_mode: string;
  last_rpc_at: string;
}

export interface HostDetail extends Host {
  host_info_xml: string;
}

export interface Preferences {
  prefs_xml: string;
  mod_time: string;
}

// ---------- Hosts API ----------

export function apiListHosts() {
  return apiFetch<Host[]>("/user/hosts");
}

export function apiGetHost(id: number) {
  return apiFetch<HostDetail>(`/user/hosts/${id}`);
}

export function apiUpdateHost(id: number, venue: string) {
  return apiFetch<void>(`/user/hosts/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ venue }),
  });
}

// ---------- Preferences API ----------

export function apiGetPreferences() {
  return apiFetch<Preferences>("/user/preferences");
}

export function apiSetPreferences(prefs_xml: string) {
  return apiFetch<void>("/user/preferences", {
    method: "PUT",
    body: JSON.stringify({ prefs_xml }),
  });
}

// ---------- Stats API ----------

export interface ProjectCredit {
  project_name: string;
  total_credit: number;
  recent_credit: number;
}

export interface UserStats {
  total_credit: number;
  recent_credit: number;
  project_count: number;
  host_count: number;
  projects: ProjectCredit[];
}

export function apiGetUserStats() {
  return apiFetch<UserStats>("/user/stats");
}

// ---------- Password Change ----------

export function apiChangePassword(old_password: string, new_password: string) {
  return apiFetch<void>("/user/change-password", {
    method: "POST",
    body: JSON.stringify({ old_password, new_password }),
  });
}

// ---------- Admin API ----------

export interface AdminUser {
  id: number;
  email: string;
  name: string;
  country: string;
  is_admin: boolean;
  created_at: string;
}

export interface AdminStats {
  total_users: number;
  total_hosts: number;
  total_projects: number;
  total_enrollments: number;
  active_sessions: number;
}

export function apiAdminListUsers() {
  return apiFetch<AdminUser[]>("/admin/users");
}

export function apiAdminGetStats() {
  return apiFetch<AdminStats>("/admin/stats");
}

export function apiAdminCreateProject(data: {
  url: string;
  name: string;
  description?: string;
  general_area?: string;
  specific_area?: string;
  home_url?: string;
}) {
  return apiFetch<{ id: number; ok: boolean }>("/admin/projects", {
    method: "POST",
    body: JSON.stringify(data),
  });
}

export function apiAdminUpdateProject(
  id: number,
  data: {
    name?: string;
    description?: string;
    general_area?: string;
    specific_area?: string;
    home_url?: string;
    is_active?: boolean;
  },
) {
  return apiFetch<void>(`/admin/projects/${id}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
}
