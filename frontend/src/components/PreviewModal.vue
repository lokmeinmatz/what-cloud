<template>
  <teleport to="#app">
    <div id="card-wrapper" @click.self="close">
      <div class="card" id="string-modal">
        <div
          class="card-header"
          style="display: grid; grid-template-columns: auto min-content"
        >
          <p style="margin: 0; vertical-align: center">
            Preview of {{ file.name }}
          </p>
          <button
            class="btn btn-outline-danger close"
            aria-label="Close Info"
            @click="close"
          >
            <span aria-hidden="true">&times;</span>
          </button>
        </div>
        <div class="card-body" style="postion: absolute">
          <img
            v-if="file.previewType == 'image'"
            :src="prevUrl"
            @load.once="upgradePreview"
          />
          <video v-else-if="file.previewType == 'video'" controls>
            <source :src="file.downloadLink()" type="video/quicktime" />
          </video>
          <h1 v-else>No preview possible</h1>
        </div>
        <button id="left" class="preview-nav" @click="goto(-1)">
          <svg
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M11 15l-3-3m0 0l3-3m-3 3h8M3 12a9 9 0 1118 0 9 9 0 01-18 0z"
            ></path>
          </svg>
        </button>
        <button id="right" class="preview-nav" @click="goto(1)">
          <svg
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 9l3 3m0 0l-3 3m3-3H8m13 0a9 9 0 11-18 0 9 9 0 0118 0z"
            ></path>
          </svg>
        </button>
      </div>
    </div>
  </teleport>
</template>


<script lang="ts">
import { defineComponent, PropType, ref } from "vue";
import { File } from "../business/fs";
import router from "../router";
import { store } from "../store";

export default defineComponent({
  name: "PreviewModal",
  props: {
    file: {
      required: true,
      type: Object as PropType<File>,
    },
  },
  emits: ["close"],
  setup(props, { emit }) {
    function close() {
      console.log("close");
      emit("close");
    }

    const prevUrl = ref(props.file.previewUrl(-1));
    const upgradePreview = () => {
      prevUrl.value = props.file.downloadLink();
    };

    function goto(offs: number) {
      if (!props.file.parent || !props.file.parent.children) {
        console.warn('no parent, kant go to neighbour')
        return
      }

      // get my index in parent children
      const myIdx = props.file.parent.children.findIndex(n => n.name == props.file.name)
      const targetIdx = myIdx + offs
      if ( targetIdx < 0 || targetIdx >= props.file.parent.children.length) {
        console.warn('reached end of children. Implement wrap arpound??')
        return
      }

      router.push(store.displayMode.value.baseUrl() + props.file.parent.children[targetIdx].path())

    }

    return {
      goto,
      close,
      prevUrl,
      upgradePreview,
    };
  },
});
</script>

<style scoped>
#card-wrapper {
  position: fixed;
  z-index: 20;
  overflow: hidden;
  display: block;
  top: 0;
  left: 0;
  height: 100vh;
  width: 100vw;
  background-color: rgba(0, 0, 0, 0.2);
}

.preview-nav {
  height: 6em;
  position: absolute;
  top: calc(50% - 3em);
  background: rgba(0, 0, 0, 0.2);
  border: rgba(255, 255, 255, 0.5) solid 2px;
}

.preview-nav svg {
  height: 3em;
  transition: transform 0.2s ease-out;
}

.preview-nav:hover svg {
  transform: scale(1.2);
}

.preview-nav#left {
  left: 0;
  border-left: none;
  border-radius: 0 2em 2em 0;
}

.preview-nav#right {
  right: 0;
  border-right: none;
  border-radius: 2em 0 0 2em;
}

.card {
  position: fixed;
  z-index: 20;
  top: 4em;
  left: 1em;
  right: 1em;
  bottom: 1em;
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

.card-body {
  padding: 0;
}

img,
video {
  width: 100%;
  height: 100%;
  object-fit: contain;
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
