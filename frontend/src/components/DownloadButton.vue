<template>
  <a :download="file.name" :href="file.downloadLink()" class="btn btn-light">
    <svg fill="none" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" stroke="currentColor"><path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path></svg>
  </a>
</template>

<script>
import { Node, pathArrayToString } from '../business/fs'

export default {
  name: 'DownloadButton',
  props: {
    file: {type: Node, required: true}
  },
  methods: {
    /**
     * @param {MouseEvent} event
     */
    download(event) {
      event.stopPropagation()
      event.preventDefault()

      const path = this.file.path()
      console.log('downloading ', path)
      window.open()
    }
  },
}
</script>

<style scoped>
</style>
