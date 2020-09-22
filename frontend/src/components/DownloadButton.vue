<template>
  <a :download="file.type == 'file' ? file.name : `${file.name}.zip`" :href="file.downloadLink()" class="btn btn-light">
    <svg fill="none" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" stroke="currentColor"><path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path></svg>
  </a>
</template>


<script lang="ts">
import { defineComponent } from 'vue'
import { Node } from '../business/fs'

export default defineComponent({
  name: 'DownloadButton',
  props: {
    file: {type: Node, required: true}
  },
  methods: {
    download(event: MouseEvent) {
      event.stopPropagation()
      event.preventDefault()

      const path = this.file.path()
      console.log('downloading ', path)
      window.open()
    }
  },
})
</script>

<style scoped>
</style>
