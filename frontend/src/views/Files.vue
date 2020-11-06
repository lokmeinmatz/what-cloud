<template>
  <div id="files" :class="{'display-node-info': nodeInfo != null}">
    <main class="container-sm">
      <div class="header">
        <PathDisplay :folder="pathDisplayObj" :mode="mode" @nodeinforequested="showNodeInfo($event)"/>
      </div>
      <div v-if="folder == 'loading'" class="loading">
        <div class="spinner-border" role="status">
          <span class="sr-only">Loading...</span>
        </div>
        <h3>Loading data...</h3>
      </div>
      <div v-else-if="folder != null">
        <FileList :folder="lastFolderLevel" @nodeinforequested="showNodeInfo($event)"/>
      </div>
      <h3 v-else>This folder doesn't exist 😥</h3>
    </main>
    <FileInfo v-model:file="nodeInfo"/>
    <div id="add-content" class="btn-group" v-if="folderIsNode && folder.isMyNode()">
      <UploadButton :folder="folder"/>
      <NewFolderButton :folder="folder"/>
    </div>
    <PreviewModal v-if="folder.type == 'file'" :file="folder" @close="goToParent()"/>
  </div>
</template>
<script lang="ts">
// eslint-disable
import { computed, defineComponent, ref, watch } from 'vue'
import FileList from '../components/FileList.vue'
import PathDisplay from '../components/PathDisplay.vue'
import FileInfo from '../components/FileInfo.vue'
import PreviewModal from '../components/PreviewModal.vue'
import UploadButton from '../components/buttons/UploadButton.vue'
import NewFolderButton from '../components/buttons/NewFolderButton.vue'
import { DisplayModeType, store } from '../store'
import { getNode, GetNodeError, Node, NodeType } from '../business/fs'
import router from '../router'


export default defineComponent({
  name: 'Files',
  components: {
    FileList,
    PathDisplay,
    FileInfo,
    UploadButton,
    NewFolderButton,
    PreviewModal
  },
  setup() {
    const folder = ref<'loading' |Node | null>('loading')
    const nodeInfo = ref<Node | null>(null)
    const mode = store.displayMode

    const folderIsNode = computed(() => {
      return folder.value instanceof Node
    })

    const lastFolderLevel = computed(() => {
      if (folder.value instanceof Node) {
        if (folder.value.type == NodeType.Folder) return folder.value
        return folder.value.parent
      }
      return null
    })

    const showNodeInfo = (node: Node) => {
      console.log('showNodeInfo', node)
      nodeInfo.value = node
    }

    const pathElements = computed<string[]>(() => {
      let r
      const mode = store.displayMode.value.mode
      if (mode == DisplayModeType.Files) {
        r = router.currentRoute.value.path.split('/').filter(e => e.trim().length > 0)
      } else 
      if (mode == DisplayModeType.Shared) {
        r = router.currentRoute.value.path.split('/').filter(e => e.trim().length > 0)
        r.shift() 
      } else { return [] }

      r.shift()
      return r
    })

    const pathDisplayObj = computed(() => {
      if (folder.value == 'loading') {
        return {pathFromRoot: pathElements.value, loading: true}
      }
      return folder.value
    })

    const goToParent = () => {
      console.log('goToParent')
      if (folder.value instanceof Node) {
        if (folder.value.parent == undefined) return
        // get last /
        const oldUrl = router.currentRoute.value.fullPath
        const lastSlash = oldUrl.lastIndexOf('/')
        
        router.push(oldUrl.substring(0, lastSlash))
      }
    }
    
    const updateFolder = async () => {
      console.log('route changed', router.currentRoute.value.path)
      try {
        
        folder.value = 'loading'
        const n = await getNode(pathElements.value)
        if (n.isOk()) {
          folder.value = n.value
        } else if (n.error == GetNodeError.NodeNotExisiting) {
          // TODO go back to start page?
          folder.value = null
        } else {
          folder.value = null
        }
        return
      }
      catch (e) {
        console.error('updateFolder() failed:', e)
        // TODO handle error better
        folder.value = 'loading'
        //this.folder = null
      }
    }

    // update folder on route change
    watch(router.currentRoute, updateFolder)


    updateFolder()
    return {
      folder,
      nodeInfo,
      showNodeInfo,
      mode,
      updateFolder,
      pathDisplayObj,
      folderIsNode,
      lastFolderLevel,
      goToParent
    }
  }
})
</script>
<style scoped>

#files {
  position: relative;
  display: grid;
  grid-template-columns: 100vw 25em;
  grid-template-rows: 100%;
  min-height: 100%;
  width: 100vw;
  align-content: stretch;
  overflow-x: hidden;
  transition: grid-template-columns ease-out 0.3s;
}

#add-content {
  position: fixed;
  right: 1em;
  bottom: 1em;
  z-index: 3;
}

#files.display-node-info {
  grid-template-columns: calc(100vw - 25em) 25em;
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
}


@media only screen and (max-width: 768px) {
  #files {
    grid-template-columns: 100vw auto;
  }

  #files.display-node-info {
    grid-template-columns: 100vw auto;
  }
}
</style>