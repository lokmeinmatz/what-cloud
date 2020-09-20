<template>
  <div class="path-display">
    <div class="btn-group" role="group" aria-label="File path">
      <router-link class="btn" to="/files">
        <svg
          v-if="mode.mode == 'files'"
          fill="none"
          style="height: 1.5em;"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
          />
        </svg>
        <svg v-else 
          style="height: 1.5em;" viewBox="0 0 20 20" fill="currentColor" class="share w-6 h-6">
          <path
            d="M15 8a3 3 0 10-2.977-2.63l-4.94 2.47a3 3 0 100 4.319l4.94 2.47a3 3 0 10.895-1.789l-4.94-2.47a3.027 3.027 0 000-.74l4.94-2.47C13.456 7.68 14.19 8 15 8z"
          />
        </svg>
      </router-link>
      <router-link
        class="btn sub-folder"
        v-for="elmt in path"
        :key="elmt.filePath"
        :to="`/files${elmt.filePath}`"
      >
        <img src="/api/static/icons/folder.svg" width="24" height="24" />
        <p>{{elmt.segment}}</p>
      </router-link>
    </div>
    <DownloadButton v-if="!folder.loading" class="download" :file="folder" />
    <button class="btn btn-primary" @click.stop="showInfo">Infos</button>
  </div>
</template>


<script lang="ts">
import { defineComponent } from 'vue'
import { Folder, Node } from "../business/fs";
import { DisplayMode } from '../store';
import DownloadButton from './DownloadButton.vue'

export default defineComponent({
  name: "PathDisplay",
  components: {
    DownloadButton
  },
  props: {
    folder: Node,
    mode: DisplayMode,
  },
  methods: {
    showInfo() {
      this.$emit('nodeinfo-requested', this.folder)
    }
  },
  computed: {
    // TODO duplicate code
    path(): Array<{segment: string; filePath: string}> {

      const res = []
      let prevPath = ''
      for (const seg of (this.folder as Folder).pathFromRoot) {
        const npath = `${prevPath}/${seg}`
        res.push({segment: seg, filePath: npath})
        prevPath = npath
      }

      return res
    }
  }
})
</script>

<style scoped>
.path-display {
  display: grid;
  grid-template-columns: min-content auto 3em 4em;
  height: 3em;
  width: 100%;
  gap: 0.5em;
}

.download {
  grid-column: 3 / span 1;
}


.btn-group .btn {
  display: flex;
  align-items: center;
  background-color: rgba(120, 120, 120, 0.1);
}

.sub-folder p {
  margin: 0;
  margin-left: 0.5em;
}

.btn-group .btn:last-child {
  background-color: rgba(120, 120, 120, 0.15);
}

.btn-group .btn:hover {
  background-color: rgba(120, 120, 150, 0.2);
}
</style>
