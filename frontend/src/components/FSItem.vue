<template>
  <div class="fs-item">
    <router-link to="">{{name}}</router-link>
  </div>
</template>

<script>
export default {
  name: 'FSItem',
  props: {
    name: String
  }
}
</script>

<style>
</style>
