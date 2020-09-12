<template>
  <div class="list-group">
    <FSItem v-for="file in content" class="list-group-item" :key="file.name" :file="file"/>
  </div>
</template>

<script>
import FSItem from './FSItem'
import { Folder, Node } from '../business/fs'

export default {
  name: 'FileList',
  components: {FSItem},
  props: {
    folder: Folder
  },
  computed: {
    content() {
      /**
       * @type {Node[]}
       */
      const c = this.folder.children
      return c.sort((a, b) => {
        if (a.type == 'folder' && b.type != 'folder') return -1 
        if (a.type != 'folder' && b.type == 'folder') return 1
        return a.name.localeCompare(b.name)
      })
    }
  }
}
</script>

<style>
</style>
