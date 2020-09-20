<template>
  <div id="files">
    <main class="container-sm">
      <div class="header">
        <PathDisplay :folder="pathDisplayObj" :mode="mode" @nodeinfo-requested="nodeInfo = $event"/>
      </div>
      <div v-if="folder == 'loading'" class="loading">
        <div class="spinner-border" role="status">
          <span class="sr-only">Loading...</span>
        </div>
        <h3>Loading data...</h3>
      </div>
      <div v-else-if="folder != null">
        <FileList :folder="folder" @nodeinfo-requested="nodeInfo = $event"/>
      </div>
      <h3 v-else>This folder doesn't exist 😥</h3>
    </main>
    <aside :class="{display: nodeInfo != null}">
      <FileInfo class="display" v-model="nodeInfo" v-if="nodeInfo != null"/>
    </aside>
  </div>
</template>
<script lang="ts">
import { defineComponent } from 'vue'
import FileList from '../components/FileList.vue'
import PathDisplay from '../components/PathDisplay.vue'
import FileInfo from '../components/FileInfo.vue'
import { store, MyFilesDisplayMode, SharedDisplayMode } from '../store'
import { getNode, Node } from '../business/fs'

export default defineComponent({
  components: {
    FileList,
    PathDisplay,
    FileInfo
  },
  async mounted() {

    this.updateFolder()
  },
  data() {
    return {
      folder: 'loading' as string | Node,
      nodeInfo: null,
      mode: store.displayMode
    }
  },
  methods: {

    async updateFolder() {
      console.log('route changed', this.$route.path)
      try {
        this.folder = 'loading'
        this.folder = await getNode(this.pathElmts)
        //console.log('successfully got new folder', this.folder)
        return
      }
      catch (e) {
        console.error('updateFolder() failed', e)
        // TODO handle error better
        this.folder = 'loading'
        //this.folder = null
      }
    }
  },
  watch: {
    
    async $route() {
      this.updateFolder()
    }
  },
  computed: {
    
    pathElmts(): string[] {
      let r
      const mode = store.displayMode.value
      if (mode instanceof MyFilesDisplayMode) {
        r = this.$route.path.split('/').filter(e => e.trim().length > 0)
      } else 
      if (mode instanceof SharedDisplayMode) {
        r = this.$route.path.split('/').filter(e => e.trim().length > 0)
        r.shift() 
      } else { return [] }

      r.shift()
      return r
    },
    pathDisplayObj() {
      if (this.folder == 'loading') {
        return {pathFromRoot: this.pathElmts, loading: true}
      }
      return this.folder
    }
  }
})
</script>
<style scoped>

#files {
  position: relative;
  display: grid;
  grid-template-columns: auto min-content;
  grid-template-rows: 100%;
  min-height: 100%;
  width: 100vw;
  align-content: stretch;
  overflow-x: hidden;
}

.header {
  margin: 1em 0;
  height: 3em;
  display: grid;
  justify-items: start;
  align-items: center;
  max-width: 100%;
  overflow-x: auto;
  position: relative;
}

aside {
  display: flex;
  transition: width ease-in-out 0.2s;
  width: 0;
}

.display {
  width: 25em;
}

@media only screen and (max-width: 768px) {
  #files {
    grid-template-columns: 1fr;
  }

  aside {
    position: fixed;
    overflow: hidden;
    justify-content: center;
    top: 0;
    height: 100vh;
    background-color: rgba(0, 0, 0, 0.1);
  }

  aside.display {
    width: 100vw;
  }
}
</style>