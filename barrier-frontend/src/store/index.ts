import { createStore } from "vuex";
import APIService from "../services/api";
import router from "@/router";

export interface State {
  api: APIService | null;
  invalidLogin: boolean;
}

// export const key: InjectionKey<Store<State>> = Symbol();

export default createStore<State>({
  state: {
    api: null,
    invalidLogin: false,
  },
  mutations: {
    loadFromLocalStorage(state) {
      const accessToken = window.localStorage.getItem("accessToken");
      const refreshToken = window.localStorage.getItem("refreshToken");

      if (accessToken !== null && refreshToken !== null) {
        state.api = new APIService({
          accessToken,
          refreshToken,
        });
        state.invalidLogin = false;
      }
    },
    login(state, api) {
      if (api.accessToken && api.refreshToken) {
        window.localStorage.setItem("accessToken", api.accessToken);
        window.localStorage.setItem("refreshToken", api.refreshToken);
      }
      state.api = api;
      state.invalidLogin = false;

      router.push("/").then();
    },
    loginFailed(state) {
      window.localStorage.removeItem("accessToken");
      window.localStorage.removeItem("refreshToken");
      state.api = null;
      state.invalidLogin = true;
    },
    logout(state) {
      window.localStorage.removeItem("accessToken");
      window.localStorage.removeItem("refreshToken");
      state.api = null;
      state.invalidLogin = false;
      router.push("/login").then();
    },
  },
  actions: {
    login(context, { login, password }: { login: string; password: string }) {
      const api = new APIService();

      api
        .login(login, password)
        .then(() => {
          context.commit("login", api);
        })
        .catch(() => context.commit("loginFailed"));
    },
    logout(context) {
      context.state.api?.logout().then(() => context.commit("logout"));
    },
  },
  modules: {},
});
