<template>
  <div class="content container-sm">
    <main>
      <h1>See all your files</h1>
      <virtual-list></virtual-list>
    </main>
    <aside></aside>
  </div>
</template>
<script>
import VirtualList from 'vue-virtual-scroll-list'

export default {
  components: { 'virtual-list': VirtualList }
}
</script>