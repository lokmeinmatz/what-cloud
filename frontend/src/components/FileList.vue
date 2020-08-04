<template>
  <div class="list-group">
    <FSItem v-for="file in content" class="list-group-item" :basePath="basePath" :key="file.name" :file="file"/>
  </div>
</template>

<script>
import FSItem from './FSItem'
import { File } from '../business/fs'

export default {
  name: 'FileList',
  components: {FSItem},
  props: {
    folder: File
  },
  computed: {
    content() {
      return this.folder.allContents()
    },
    basePath() {
      return this.$route.path
    }
  }
}
</script>

<style>
</style>
