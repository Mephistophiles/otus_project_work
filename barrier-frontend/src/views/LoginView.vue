<template>
  <div
    class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8"
  >
    <div class="max-w-md w-full space-y-8">
      <div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
          Sign in to your account
        </h2>
      </div>
      <form class="mt-8 space-y-6" @submit="doLogin" @keypress.enter="doLogin">
        <input type="hidden" name="remember" value="true" />
        <div class="rounded-md shadow-sm -space-y-px">
          <div>
            <label for="email-address" class="sr-only">Login or Email</label>
            <input
              id="email-address"
              name="email"
              type="email"
              v-model="email"
              autocomplete="email"
              required
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              placeholder="Login or Email"
            />
          </div>
          <div>
            <label for="password" class="sr-only">Password</label>
            <input
              id="password"
              name="password"
              type="password"
              v-model="password"
              autocomplete="current-password"
              required
              class="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
              placeholder="Password"
            />
          </div>
        </div>

        <span
          v-if="$store.state.invalidLogin"
          class="flex items-center font-medium tracking-wide text-red-500 text-xs mt-1 ml-1"
        >
          Invalid login or password!
        </span>

        <div>
          <button
            type="button"
            @click="doLogin"
            :class="{
              'opacity-50': invalidLogin,
              'cursor-not-allowed': invalidLogin,
            }"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-dlink-main hover:bg-dlink-hover disabled:opacity-10 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <span class="absolute left-0 inset-y-0 flex items-center pl-3">
              <svg
                class="h-5 w-5 text-white-500 group-hover:text-white-400"
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
                aria-hidden="true"
              >
                <path
                  fill-rule="evenodd"
                  d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z"
                  clip-rule="evenodd"
                />
              </svg>
            </span>
            Sign in
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "LoginView",
  components: {},
  data() {
    return {
      email: "",
      password: "",
    };
  },
  computed: {
    invalidLogin: function (): boolean {
      return (
        this.email.trim().length === 0 || this.password.trim().length === 0
      );
    },
    login: function (): string {
      return this.email.split("@")[0];
    },
  },
  methods: {
    doLogin: function () {
      if (this.invalidLogin) {
        return;
      }

      this.$store.dispatch("login", {
        login: this.login,
        password: this.password,
      });
    },
  },
});
</script>
