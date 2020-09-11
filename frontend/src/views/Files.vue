<template>
  <div id="files">
    <main class="container-sm">
      <div class="header">
        <PathDisplay :folder="pathDisplayObj" :mode="mode"/>
      </div>
      <div v-if="folder == 'loading'" class="loading">
        <div class="spinner-border" role="status">
          <span class="sr-only">Loading...</span>
        </div>
        <h3>Loading data...</h3>
      </div>
      <div v-else-if="folder != null">
        <FileList :folder="folder"/>
      </div>
      <h3 v-else>This folder doesn't exist 😥</h3>
    </main>
    <aside :class="{display: nodeInfo != null}">
      <FileInfo class="display" :file="nodeInfo" v-if="nodeInfo != null"/>
    </aside>
  </div>
</template>
<script>
import FileList from '../components/FileList'
import PathDisplay from '../components/PathDisplay'
import FileInfo from '../components/FileInfo'
import { getNode } from '../business/fs'
import { state } from '../business/globalState'

export default {
  components: {
    FileList,
    PathDisplay,
    FileInfo
  },
  async mounted() {
    state.nodeInfoDisplay.subscribeWithId('files', f => {
      if (f.fetched) this.nodeInfo = f
    })

    state.fileDisplayState.subscribeWithId('files', s => {
      this.mode = s
    })

    this.updateFolder()
  },
  data() {
    return {
      folder: 'loading',
      nodeInfo: null,
      mode: state.fileDisplayState.currentValue()
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
    pathDisplayObj() {
      if (this.folder == 'loading') {
        return {pathFromRoot: this.pathElmts, loading: true}
      }
      return this.folder
    },
    pathElmts() {
      let r
      switch (state.fileDisplayState.currentValue().mode) {
        case 'files':
          r = this.$route.path.split('/').filter(e => e.trim().length > 0)
          break
        case 'shared':
          r = this.$route.path.split('/').filter(e => e.trim().length > 0)
          r.shift()
          break
        default:
          return []
      }
      r.shift()
      return r
    }
  }
}
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