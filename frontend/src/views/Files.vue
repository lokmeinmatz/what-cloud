<template>
  <div class="content container-sm">
    <main>
      <div class="header">
        <div class="btn-group" role="group" aria-label="File path">
          <router-link class="btn" to="/files">
            <svg fill="none" style="height: 1.5em;" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" stroke="currentColor"><path d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path></svg>
          </router-link>
          <router-link 
          class="btn"
            v-for="elmt in subPath"
            :key="elmt.filePath"
            :to="`/files${elmt.filePath}`"
            >
          ▶ {{elmt.segment}}
          </router-link> 
        </div>
      </div>
      <div v-if="folder != null">
        <FileList :folder="folder"/>
      </div>
      <h3 v-else>This folder doesn't exist 😥</h3>
    </main>
    <aside></aside>
  </div>
</template>
<script>
import FileList from '../components/FileList'
import { getFolder } from '../business/fs'

export default {
  components: {
    FileList
  },
  async mounted() {
    this.updateFolder()
  },
  data() {
    return {
      folder: null
    }
  },
  methods: {
    async updateFolder() {
      console.log('route changed', this.$route.path)
      console.log(this.subPath)
      try {
        this.folder = await getFolder(this.pathElmts)
        //console.log('successfully got new folder', this.folder)
        return
      }
      catch (e) {
        console.error(e)
        this.folder = null
      }
    }
  },
  watch: {
    async $route() {
      this.updateFolder()
    }
  },
  computed: {
    pathElmts() {
      return this.$route.path.substr(7).split('/').filter(e => e.trim().length > 0)
    },
    subPath() {
  
      return this.pathElmts.reduce(([collector, prevPath], curr) => {
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
<style scoped>
.btn {
  display: flex;
  align-items: flex-end;
  background-color: rgba(120, 120, 120, 0.1);
}

.btn:last-child {
  background-color: rgba(120, 120, 120, 0.15);
}

.btn:hover {
  background-color: rgba(120, 120, 150, 0.2);
}

.header {
  padding: 1em 0;
  height: 4em;
  display: grid;
  justify-items: start;
  align-items: center;
}
</style>