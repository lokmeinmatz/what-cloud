<template>
  <div :class="{'fs-item':true, 'preview':preview, 'card':preview}">
    <img
      v-if="!preview"
      :src="`/api/static/icons/${file.type == 'file' ? file.ext() : 'folder'}.svg`"
      width="24"
      height="24"
      style="margin-right: 0.5em;"
    />
    <div v-else ref="prevImg" class="preview-img" :style="{'background-image': prevImgUrl}"></div>
    <router-link v-if="file.type == 'folder'" class="f-name above" :to="displayMode.baseUrl() + file.path()">{{file.name}}</router-link>
    <a v-else class="f-name above" :href="file.downloadLink()">{{file.name}}</a>
    <div class="options">
      <DownloadButton class="above" :file="file" />
      <button class="btn btn-primary above" @click.stop="showInfo($event, file)">Infos</button>
    </div>
  </div>
</template>


<script lang="ts">
import { computed, defineComponent } from 'vue'
import { Node } from "../business/fs";
import DownloadButton from "./buttons/DownloadButton.vue";
import { store } from '../store'

export default defineComponent({
  name: "FSItem",
  components: {
    DownloadButton,
  },
  props: {
    file: Node,
    preview: Boolean
  },

  setup(props, ctx) {

    function showInfo(event: MouseEvent, file: Node) {
      event.stopPropagation();
      event.preventDefault();
      ctx.emit('nodeinfo-requested', file)
    }

    const prevImgUrl = computed<string>(() => {
      return `url("/api/preview/file?path=${encodeURIComponent(props.file?.path()??"unknown")}&token=${store.user.value?.authToken}&resolution=256")`
    })


    return {
      displayMode: store.displayMode,
      showInfo,
      prevImgUrl
    }
  } 

  
})
</script>

<style scoped>
.fs-item a {
  text-align: left;
}

.dark .fs-item a:hover {
  color: var(--cyan);
}

.fs-item {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) 3em 4em;
  gap: 0.5em;
  align-items: center;
  height: 3em;
  padding: 0.5em;
  background-color: transparent;
  border: 1px solid;
  border-color: rgba(128, 128, 128, 0.2);
  overflow: hidden;
}

.fs-item.preview {
  /*random*/
  margin: 0.1em;
  width: 10em;
  height: 10em;
  position: relative;
}

.options {
  display: grid;
  grid-template-columns: 3em 4em; 
  gap: 0.2em;
  position: absolute;
  /* bottom: 0.5em;
  top: 0.5em; */
  right: 0.5em;
}

.fs-item.preview .options {
  bottom: 0.5em;
}


.fs-item.preview .above {
  z-index: 2;
}

.fs-item.preview .preview-img {
  background-color: transparent;
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 1;
  background-position: 50% 50%;
  opacity: 0.2;
  transition: opacity 0.2s linear;
}

.fs-item.preview:hover .preview-img {
  opacity: 1;
}

.fs-item.preview  .f-name {
  position: absolute;
  top: 0.5em;
  left: 0.5em;
}

.fs-item:hover {
  background-color: rgba(128, 128, 128, 0.1);
}

.f-name {
  margin: 0;
  word-wrap: break-word;
  word-wrap: break-all;
  max-width: 100%;
}
</style>
