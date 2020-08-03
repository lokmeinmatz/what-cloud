<template>
  <div class="fs-item">
    <img :src="`/static/icons/${file.type == 'file' ? file.ext : 'folder'}`" width="24" height="24" style="margin-right: 0.5em;"/>
    <router-link :to="`${basePath}/${file.name}`">{{file.name}}</router-link>
  </div>
</template>

<script>
export default {
  name: 'FSItem',
  props: {
    basePath: String,
    file: Object
  }
}
</script>

<style>
</style>
