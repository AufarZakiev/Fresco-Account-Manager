<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";

const auth = useAuthStore();
const router = useRouter();

const name = ref("");
const email = ref("");
const password = ref("");
const confirmPassword = ref("");
const submitting = ref(false);
const errorMsg = ref<string | null>(null);

async function onSubmit() {
  errorMsg.value = null;

  if (password.value !== confirmPassword.value) {
    errorMsg.value = "Passwords do not match";
    return;
  }

  submitting.value = true;

  const ok = await auth.register(email.value, password.value, name.value);

  if (ok) {
    router.push("/");
  } else {
    errorMsg.value = auth.error ?? "Registration failed";
  }

  submitting.value = false;
}
</script>

<template>
  <div class="auth-page">
    <form class="auth-form" @submit.prevent="onSubmit">
      <h1>Create Account</h1>

      <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

      <label>
        Name
        <input
          v-model="name"
          type="text"
          required
          autocomplete="name"
          placeholder="Your name"
        />
      </label>

      <label>
        Email
        <input
          v-model="email"
          type="email"
          required
          autocomplete="email"
          placeholder="you@example.com"
        />
      </label>

      <label>
        Password
        <input
          v-model="password"
          type="password"
          required
          autocomplete="new-password"
          placeholder="Password"
          minlength="6"
        />
      </label>

      <label>
        Confirm Password
        <input
          v-model="confirmPassword"
          type="password"
          required
          autocomplete="new-password"
          placeholder="Confirm password"
        />
      </label>

      <button type="submit" :disabled="submitting" class="btn-primary">
        {{ submitting ? "Creating account..." : "Register" }}
      </button>

      <p class="auth-alt">
        Already have an account?
        <RouterLink to="/login">Sign In</RouterLink>
      </p>
    </form>
  </div>
</template>
