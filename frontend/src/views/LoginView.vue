<script setup lang="ts">
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useAuthStore } from "../stores/auth";

const auth = useAuthStore();
const router = useRouter();
const route = useRoute();

const email = ref("");
const password = ref("");
const submitting = ref(false);
const errorMsg = ref<string | null>(null);

async function onSubmit() {
  submitting.value = true;
  errorMsg.value = null;

  const ok = await auth.login(email.value, password.value);

  if (ok) {
    const redirect =
      typeof route.query.redirect === "string"
        ? route.query.redirect
        : "/";
    router.push(redirect);
  } else {
    errorMsg.value = auth.error ?? "Login failed";
  }

  submitting.value = false;
}
</script>

<template>
  <div class="auth-page">
    <form class="auth-form" @submit.prevent="onSubmit">
      <h1>Sign In</h1>

      <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

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
          autocomplete="current-password"
          placeholder="Password"
        />
      </label>

      <button type="submit" :disabled="submitting" class="btn-primary">
        {{ submitting ? "Signing in..." : "Sign In" }}
      </button>

      <p class="auth-alt">
        Don't have an account?
        <RouterLink to="/register">Register</RouterLink>
      </p>
    </form>
  </div>
</template>
