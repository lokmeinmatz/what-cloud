<template>
  <div class="btn-group" role="group" aria-label="File path">
    <router-link class="btn" to="/files">
      <svg
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
</template>

<script>
import { Node } from "../business/fs";

export default {
  name: "PathDisplay",
  props: {
    path: Array
  }
};
</script>

<style scoped>
.btn {
  display: flex;
  align-items: center;
  background-color: rgba(120, 120, 120, 0.1);
}

.sub-folder p {
  margin: 0;
  margin-left: 0.5em;
}

.btn:last-child {
  background-color: rgba(120, 120, 120, 0.15);
}

.btn:hover {
  background-color: rgba(120, 120, 150, 0.2);
}
</style>
