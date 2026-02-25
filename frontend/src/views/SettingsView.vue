<script setup lang="ts">
import { ref } from "vue";
import { useAuthStore } from "../stores/auth";
import { apiChangePassword, ApiError } from "../api/client";

const auth = useAuthStore();

const oldPassword = ref("");
const newPassword = ref("");
const confirmPassword = ref("");
const submitting = ref(false);
const errorMsg = ref<string | null>(null);
const successMsg = ref<string | null>(null);

async function onChangePassword() {
  errorMsg.value = null;
  successMsg.value = null;

  if (newPassword.value !== confirmPassword.value) {
    errorMsg.value = "New passwords do not match.";
    return;
  }

  if (newPassword.value.length < 6) {
    errorMsg.value = "New password must be at least 6 characters.";
    return;
  }

  submitting.value = true;
  try {
    await apiChangePassword(oldPassword.value, newPassword.value);
    successMsg.value = "Password changed successfully.";
    oldPassword.value = "";
    newPassword.value = "";
    confirmPassword.value = "";
  } catch (e: unknown) {
    if (e instanceof ApiError) {
      errorMsg.value = e.message || "Failed to change password.";
    } else {
      errorMsg.value = "Failed to change password.";
    }
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="page">
    <div class="page-header">
      <h1 class="page-title">Settings</h1>
    </div>

    <div class="card settings-section">
      <h2>Account Information</h2>
      <dl class="info-list">
        <dt>Name</dt>
        <dd>{{ auth.user?.name }}</dd>
        <dt>Email</dt>
        <dd>{{ auth.user?.email }}</dd>
        <dt>Country</dt>
        <dd>{{ auth.user?.country || "Not set" }}</dd>
      </dl>
    </div>

    <div class="card settings-section">
      <h2>Change Password</h2>

      <div v-if="successMsg" class="success-banner">{{ successMsg }}</div>
      <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

      <form class="password-form" @submit.prevent="onChangePassword">
        <label>
          Current Password
          <input
            v-model="oldPassword"
            type="password"
            required
            autocomplete="current-password"
            placeholder="Enter current password"
          />
        </label>

        <label>
          New Password
          <input
            v-model="newPassword"
            type="password"
            required
            autocomplete="new-password"
            placeholder="Enter new password"
          />
        </label>

        <label>
          Confirm New Password
          <input
            v-model="confirmPassword"
            type="password"
            required
            autocomplete="new-password"
            placeholder="Confirm new password"
          />
        </label>

        <button type="submit" class="btn-primary" :disabled="submitting">
          {{ submitting ? "Changing..." : "Change Password" }}
        </button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.settings-section {
  margin-bottom: var(--space-lg);
}

.password-form {
  max-width: 400px;
}

.password-form .btn-primary {
  margin-top: var(--space-sm);
}
</style>
