<template>
  <router-view />
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "App",
  mounted: function () {
    this.$store.commit("loadFromLocalStorage");
  },
});
</script>
