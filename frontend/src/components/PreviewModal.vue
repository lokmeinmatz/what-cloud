<template>
  <teleport to="#app">
    <div id="card-wrapper" @click.self="close">
      <div class="card" id="string-modal">
        <div
          class="card-header"
          style="display: grid; grid-template-columns: auto min-content"
        >
          <p style="margin: 0; vertical-align: center">Preview of {{file.name}}</p>
          <button
            class="btn btn-outline-danger close"
            aria-label="Close Info"
            @click="close"
          >
            <span aria-hidden="true">&times;</span>
          </button>
        </div>
        <div class="card-body" style="postion: absolute">
          <img v-if="file.previewType == 'image'" :src="prevUrl" @load.once="upgradePreview"/>
          <video v-else-if="file.previewType == 'video'" controls>
            <source :src="file.downloadLink()" type="video/quicktime">
          </video>
          <h1 v-else>No preview possible</h1>
        </div>
      </div>
    </div>
  </teleport>
</template>


<script lang="ts">
import { defineComponent, PropType, ref } from "vue";
import { File } from "../business/fs";

export default defineComponent({
  name: "PreviewModal",
  props: {
    file: {
      required: true,
      type: Object as PropType<File>
    }
  },
  setup(props, {emit}) {
    function close() {
      console.log('close')
      emit('close')
    }

    const prevUrl = ref(props.file.previewUrl(-1))
    const upgradePreview = () => {
      prevUrl.value = props.file.downloadLink()
    }
    return {
      close,
      prevUrl,
      upgradePreview
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

img, video {
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
