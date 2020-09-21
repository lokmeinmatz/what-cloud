<template>
  <div class="fs-item">
    <img
      :src="`/api/static/icons/${file.type == 'file' ? file.ext() : 'folder'}.svg`"
      width="24"
      height="24"
      style="margin-right: 0.5em;"
    />
    <router-link v-if="file.type == 'folder'" class="f-name" :to="'/files' + file.path()">{{file.name}}</router-link>
    <a v-else class="f-name" :href="file.downloadLink()">{{file.name}}</a>
    <DownloadButton :file="file" />
    <button class="btn btn-primary" @click.stop="showInfo($event, file)">Infos</button>
  </div>
</template>


<script lang="ts">
import { defineComponent } from 'vue'
import { Node } from "../business/fs";
import DownloadButton from "./DownloadButton.vue";

export default defineComponent({
  name: "FSItem",
  components: {
    DownloadButton,
  },
  props: {
    file: Node,
  },
  methods: {
    showInfo(event: MouseEvent, file: Node) {
      event.stopPropagation();
      event.preventDefault();
      this.$emit('nodeinfo-requested', file)
    },
  },
})
</script>

<style scoped>
.fs-item a {
  text-align: left;
}
.fs-item {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) 3em 4em;
  gap: 0.5em;
  align-items: center;
  min-height: 2em;
  padding: 0.5em;
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
