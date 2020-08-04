<template>
  <router-link :to="`${basePath}/${file.name}`" class="fs-item">
    <img :src="`/api/static/icons/${file.type == 'file' ? file.ext : 'folder'}.svg`" width="24" height="24" style="margin-right: 0.5em;"/>
    <p class="f-name">{{file.name}}</p>
    <button class="btn btn-primary" @click.stop="showInfo($event, file)">Infos</button>
  </router-link>
</template>

<script>
import { File } from '../business/fs'

export default {
  name: 'FSItem',
  props: {
    basePath: String,
    file: Object
  },
  methods: {
    /**
     * @param {MouseEvent} event
     * @param {File} file
     */
    showInfo(event, file) {
      event.stopPropagation()
      event.preventDefault()

      this.$store.commit('displayFileInfo', file)
    }
  }
}
</script>

<style scoped>
.fs-item {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) 4em;
  align-items: center;
  min-height: 2em;
  padding: 0.5em;
}

.fs-item:hover {
  background-color: rgba(128, 128, 128, 0.1);
}

.f-name {
  margin: 0 0 0 1em;
  word-wrap: break-word;
  word-wrap: break-all;
  max-width: 100%;
}
</style>
