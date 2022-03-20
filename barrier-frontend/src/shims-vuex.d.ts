import { State } from "@/store";
import { Store } from "vuex";

/* eslint-disable */
declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module "@vue/runtime-core" {
  // Declare your own store states.
  interface ComponentCustomProperties {
    $store: Store<State>;
  }
}
