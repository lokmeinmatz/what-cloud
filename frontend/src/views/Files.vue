<template>
  <div class="content container-sm">
    <main>
      <h1 v-if="isRoot">All files</h1>
      <div v-else class="btn-group" role="group" aria-label="File path">
        <router-link 
          v-for="(elmt, index) in subPath"
          :key="elmt.filePath"
          :to="`/files${elmt.filePath}`"
          :class="['btn',  (index + 1 != subPath.length) ? 'btn-secondary' : 'btn-primary']"
          >
        ▶ {{elmt.segment}}
        </router-link> 
      </div>
    </main>
    <aside></aside>
  </div>
</template>
<script>

export default {
  mounted() {
    console.log(this.$route.path.substr(6))
  },
  data() {
    return {
      files: []
    }
  },
  computed: {
    subPath() {
      const segments = this.$route.path.substr(7).split('/').filter(e => e.trim().length > 0)
      if (segments.length == 0) return []
      return segments.reduce(([collector, prevPath], curr) => {
        const npath = `${prevPath}/${curr}`
        collector.push({segment: curr, filePath: npath})
        return [collector, npath]
      }, [[], ''])[0]
    },

    isRoot() {
      return this.subPath.length == 0
    }
  }
}
</script>