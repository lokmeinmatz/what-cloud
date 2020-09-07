<template>
  <div class="card" id="file-info" v-if="file != null">
    <div class="card-header" style="display: grid; grid-template-columns: auto min-content;">
      <p style="margin: 0; vertical-align: center;">File Info</p>
      <button class="btn btn-outline-danger" style="padding: 0.5em; display: grid;" @click="close">
        <svg width="10" height="10">
          <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" />
          <line x1="0" y1="10" x2="10" y2="0" stroke="currentColor" />
        </svg>
      </button>
    </div>
    <div class="card-body" style="postion:absolute;">
      <div class="card-title">
        <img :src="`/api/static/icons/${file.type == 'folder' ? 'folder' : file.ext()}.svg`" />
        <h4 style="text-wrap: break-word; width: 100%">{{file.name}}</h4>
      </div>
      <table class="table" style="position: relative;">
        <div id="meta-loader" v-if="fetchingMeta">
          <div class="spinner-border" role="status">
            <span class="sr-only">Loading...</span>
          </div>
        </div>
        <tbody>
          <tr>
            <td>Size</td>
            <td>{{ fileSize }}</td>
          </tr>
          <tr>
            <td>Modified</td>
            <td>{{ file.lastModified }}</td>
          </tr>
          <tr>
            <td>Share</td>
            <td>
              <input class="form-check-input" type="checkbox" value id="share-this" />
              <label class="form-check-label" for="share-this">Share this folder</label>
            </td>
          </tr>
          <tr>
            <td>Shared-URL<br/>Click to copy</td>
            <td>
              <a href="">shared-link</a>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
import FSItem from "./FSItem";
import { Node } from "../business/fs";
import { ByteToFormattedString } from "../business/utils";
import { state } from "../business/globalState";

export default {
  name: "FileInfo",
  data() {
    return {
      fetchingMeta: false,
    };
  },
  async mounted() {
    await this.loadMeta();
  },
  watch: {
    file: async function (newFile) {
      await this.loadMeta();
    },
  },
  props: {
    file: Object,
  },
  methods: {
    async loadMeta() {
      if (!this.file.fetched) {
        this.fetchingMeta = true;
        await this.file.loadMetadata();
        this.fetchingMeta = false;
      }
    },
    close() {
      state.nodeInfoDisplay.emit(null);
    },
  },
  computed: {
    fileSize() {
      return (
        (this.file.type == "folder" ? "≥" : "") +
        (this.file.fetched ? ByteToFormattedString(this.file.size) : "unknown")
      );
    },
  },
};
</script>

<style scoped>
#file-info {
  align-self: center;
  /*width: 30em;*/
  max-width: 90vw;
}
.card-title {
  display: grid;
  grid-template-columns: 3em minmax(0, 1fr);
  width: 100%;
  align-items: center;
}

.card-title * {
  margin: 0;
}

.card-title img {
  width: 2em;
}

#meta-loader {
  position: absolute;
  width: 100%;
  height: 100%;
  background: rgba(100, 100, 100, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
