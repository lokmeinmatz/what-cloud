<template>
  <div id="files">
    <main class="container-sm">
      <div class="header">
        <PathDisplay :path="subPath"/>
      </div>
      <div v-if="folder != null">
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
import { getFolder } from '../business/fs'
import { state } from '../business/globalState'

export default {
  components: {
    FileList,
    PathDisplay,
    FileInfo
  },
  async mounted() {
    state.nodeInfoDisplay.subscribeWithId('files', f => {
      this.nodeInfo = f
    })
    this.updateFolder()
  },
  data() {
    return {
      folder: null,
      nodeInfo: null
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
}

aside {
  display: flex;
  transition: width ease-in-out 0.2s;
  width: 0;
}

.display {
  width: 20em;
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