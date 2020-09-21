<template>
  <div class="list-group">
    <FSItem v-for="file in content" class="list-group-item" :key="file.name" :file="file" @nodeinfo-requested="$emit('nodeinfo-requested', $event)"/>
  </div>
</template>


<script lang="ts">
import { defineComponent } from 'vue'
import FSItem from './FSItem.vue'
import { Folder, Node } from '../business/fs'

export default defineComponent({
  name: 'FileList',
  components: {FSItem},
  props: {
    folder: Folder
  },
  computed: {
    content(): Node[] {
      const c = (this.folder as Folder).children
      if (c == undefined) return []
      return c.sort((a, b) => {
        if (a.type == 'folder' && b.type != 'folder') return -1 
        if (a.type != 'folder' && b.type == 'folder') return 1
        return a.name.localeCompare(b.name)
      })
    }
  }
})
</script>

<style>
</style>
