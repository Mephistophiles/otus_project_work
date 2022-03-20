import { createRouter, createWebHistory, RouteRecordRaw } from "vue-router";
import HomeView from "../views/HomeView.vue";
import store from "../store/";

const LoginView = () => import("../views/LoginView.vue");

const routes: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "HomeView",
    component: HomeView,
  },
  {
    path: "/login",
    name: "LoginView",
    component: LoginView,
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

router.beforeEach((to, _from, next) => {
  if (store.state.api === null && to.name !== "LoginView") {
    next({ name: "LoginView" });
  } else if (store.state.api !== null && to.name === "LoginView") {
    next({ name: "HomeView" });
  } else {
    next();
  }
});

export default router;
