<template>
  <div class="card" id="file-info" v-if="file != null">
    <div class="card-header" style="display: grid; grid-template-columns: auto min-content;">
      <p style="margin: 0; vertical-align: center;">File Info</p>
      <button class="btn btn-outline-danger" style="padding: 0.5em; display: grid;" @click="close">
        <svg width="10" height="10">
          <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor"/>
          <line x1="0" y1="10" x2="10" y2="0" stroke="currentColor"/>
        </svg>
      </button>
    </div>
    <div class="card-body">
      <div class="card-title">
        <img :src="`/api/static/icons/${file.type == 'folder' ? 'folder' : file.ext}.svg`" />
        <h4>{{file.name}}</h4>
      </div>
    </div>
  </div>
</template>

<script>
import FSItem from "./FSItem";
import { File } from "../business/fs";

export default {
  name: "FileInfo",
  props: {
    file: Object,
  },
  methods: {
    close() {
      this.$store.commit('displayFileInfo', null)
    }
  },
};
</script>

<style scoped>
#file-info {
  align-self: center;
  width: 20em;
}

@media screen {
}

.card-title {
  display: grid;
  grid-template-columns: 3em auto;
  align-items: center;
}

.card-title * {
  margin: 0;
}

.card-title img {
  width: 2em;
}
</style>
