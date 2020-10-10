<template>
  <teleport to="#app">
    <transition name="flyin">
      <div id="card-wrapper" v-if="visible" @click.self="close">
        <div class="card" id="string-modal">
          <div
            class="card-header"
            style="display: grid; grid-template-columns: auto min-content"
          >
            <p style="margin: 0; vertical-align: center">{{ config.title }}</p>
            <button
              class="btn btn-outline-danger close"
              aria-label="Close Info"
              @click="close"
            >
              <span aria-hidden="true">&times;</span>
            </button>
          </div>
          <div class="card-body" style="postion: absolute">
            <input type="text" name="text" id="text" v-model="text">
            <button class="btn btn-success" @click="submit">{{ config.confirmText }}</button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>


<script lang="ts">
import { defineComponent, PropType, ref } from "vue";

interface ModalProp {
  title: string;
  confirmText: string;
}

export default defineComponent({
  name: "StringInsertModal",
  props: {
    config: { required: true, type: Object as PropType<ModalProp> },
  },
  setup() {
    const visible = ref(false);
    const text = ref("");
    const reject = ref<(() => void) | null>(null);
    const resolve = ref<((entered: string) => void) | null>(null);

    async function showModal(startString?: string): Promise<string> {
      text.value = startString || "";
      visible.value = true;
      return new Promise((res, rej) => {
        if (reject.value != null) {
          console.warn("showModal didn't return last call. rejecting last");
          reject.value();
        }
        resolve.value = res;
        reject.value = rej;
      });
    }

    function close() {
      visible.value = false;
      reject.value?.();
      reject.value = null;
    }

    function submit() {
      const t = text.value;
      visible.value = false;
      resolve.value?.(t);
      resolve.value = null;
      reject.value = null;
    }

    return {
      visible,
      close,
      text,
      showModal,
      submit
    };
  },
});
</script>

<style scoped>
#card-wrapper {
  position: fixed;
  overflow: hidden;
  display: flex;
  justify-content: center;
  top: 0;
  left: 0;
  height: 100vh;
  width: 100vw;
  background-color: rgba(0, 0, 0, 0.2);
}

.card {
  align-self: center;
  width: 100%;
  max-width: min(90vw, 20em);
}

.close {
  padding: 0;
  width: 1em;
  height: 1em;
}

.dark .card {
  background-color: #111;
}

.dark .card-header {
  background-color: #222;
}

#text {
  border-radius: 0.25rem;
  border: 2px rgba(100, 100, 100, 0.5) solid;
  padding: 0.25em;
  margin-bottom: 1em;
}

.dark #text {
  background-color: #222;
}

.toast-container {
  position: fixed;
  bottom: 1em;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.toast {
  width: min-content;
}

.toast-header {
  justify-content: center;
}

.dark #sharedLink {
  background-color: #111;
  border-radius: 0.2em;
  border: #444 2px solid;
}

#meta-loader {
  position: absolute;
  width: 100%;
  height: 100%;
  background: rgba(100, 100, 100, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

.flyin-enter-active,
.flyin-leave-active {
  transition: transform 0.2s ease-out;
}

.flyin-enter-from,
.flyin-leave-to {
  transform: translateX(100vw);
}
.flyin-enter-to,
.flyin-leave-from {
  transform: translateX(0vw);
}
</style>
