<template>
  <button class="btn" @click="openSelector">
    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path></svg>  
    <input type="file" ref="hiddenInput" style="display: none;" multiple accept="*" @change="handleUpload"/>
  </button>
</template>


<script lang="ts">
import { defineComponent } from 'vue'
import { Node } from '../business/fs'
import { uploadFiles } from '../business/upload'

export default defineComponent({
  name: 'UploadButton',
  props: {
    folder: {type: Node, required: true}
  },
  methods: {
    openSelector() {
      (this.$refs.hiddenInput as HTMLInputElement).click()
    },
    async handleUpload(evt: Event) {
      if (evt.target instanceof HTMLInputElement && evt.target.files != null)
        uploadFiles(this.folder.pathFromRoot, evt.target.files)
    }
  },
})
</script>

<style scoped>
.btn {
  background-color: var(--primary);
  width: 3em;
  height: 3em;
  padding: 0.2em;
  color: white;
}

.btn:hover {
  background-color: rgba(128, 128, 128, 0.3);
}
</style>
