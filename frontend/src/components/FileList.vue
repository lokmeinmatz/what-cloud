<template>
  <div class="list-group">
    <FSItem v-for="file in content" class="list-group-item" :key="file.name" :file="file"/>
  </div>
</template>

<script>
import FSItem from './FSItem'
import { Folder } from '../business/fs'

export default {
  name: 'FileList',
  components: {FSItem},
  props: {
    folder: Folder
  },
  computed: {
    content() {
      return this.folder.children
    }
  }
}
</script>

<style>
</style>
