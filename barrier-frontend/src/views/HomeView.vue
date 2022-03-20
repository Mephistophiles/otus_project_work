<template>
  <NavBar />
  <div class="container mx-auto">
    <span v-if="emptyGates" class="text-bold text-3xl inline-block align-middle"
      >No gates available</span
    >
    <div class="flex flex-col">
      <div class="-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
        <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
          <div
            class="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg"
          >
            <table class="min-w-full divide-y divide-gray-200">
              <tbody class="bg-white divide-y divide-gray-200">
                <tr v-for="gate in gates" :key="gate.name">
                  <td class="px-6 py-4">
                    <div class="flex items-center">
                      <div class="ml-4 text-sm">
                        {{ gate.description }}
                      </div>
                    </div>
                  </td>
                  <td
                    class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium"
                  >
                    <LockButton @click="openGate(gate.name)" />
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import NavBar from "@/components/NavBar.vue";
import LockButton from "@/components/LockButton.vue";

import { defineComponent } from "vue";
import { IGateList } from "@/services/api";

export default defineComponent({
  name: "HomeView",
  data() {
    return {
      gates: Array<IGateList>(),
    };
  },
  mounted: async function () {
    const gates = await this.$store.state.api?.getGates();

    if (gates !== undefined) {
      this.gates = gates;
    }
  },
  computed: {
    emptyGates(): boolean {
      return this.gates.length === 0;
    },
  },
  components: {
    NavBar,
    LockButton,
  },
  methods: {
    async openGate(gate: string) {
      await this.$store.state.api?.openGate(gate);
    },
  },
});
</script>
