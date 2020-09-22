<template>
  <transition name="flyin">
    <div id="f-info-wrapper" v-if="file != null" @click.self="close">
      <aside class="card" id="file-info">
        <div class="card-header" style="display: grid; grid-template-columns: auto min-content;">
          <p style="margin: 0; vertical-align: center;">File Info</p>
          <button
            class="btn btn-outline-danger"
            style="padding: 0.5em; display: grid;"
            @click="close"
          >
            <svg width="10" height="10">
              <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" />
              <line x1="0" y1="10" x2="10" y2="0" stroke="currentColor" />
            </svg>
          </button>
        </div>
        <div class="card-body" style="postion:absolute;">
          <div class="card-title">
            <img :src="`/api/static/icons/${file.type == 'file' ? file.ext() : 'folder'}.svg`" />
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
                  <input
                    class="form-check-input"
                    type="checkbox"
                    file
                    id="share-this"
                    v-model="sharedToggle"
                  />
                  <label class="form-check-label" for="share-this">Share this folder</label>
                </td>
              </tr>
              <tr v-if="file.shared">
                <td>Shared-URL</td>
                <td>
                  <input
                    @click="copyShared"
                    readonly
                    type="text"
                    ref="sharedLink"
                    :value="file.sharedLink()"
                    data-toggle="tooltip"
                    title="Click to copy"
                  />
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </aside>
    </div>
  </transition>
</template>


<script lang="ts">
import { computed, defineComponent, ref, PropType, watch } from "vue";
import { Node } from "../business/fs";
import { ByteToFormattedString } from "../business/utils";

export default defineComponent({
  name: "FileInfo",
  props: {
    file: { type: Object as PropType<Node> },
  },
  setup(props, { emit }) {
    const fetchingMeta = ref(false);
    async function fetchFileMeta() {
      if (props.file != undefined) {
        if (!props.file.fetched) {
          console.log("uncached, get data");
          fetchingMeta.value = true;
          await props.file.fetch();
          fetchingMeta.value = false;
        }
      }
    }
    console.log("added watcher");
    watch(props, fetchFileMeta);

    fetchFileMeta();

    function close() {
      emit("update:file", null);
    }

    const fileSize = computed<string>(() => {
      const v = props.file as Node;
      return (
        (v.type == "folder" ? "≥" : "") +
        (v.fetched ? ByteToFormattedString(v.size) : "unknown")
      );
    });

    const sharedToggle = computed({
      get(): boolean {
        return (props.file as Node).shared != null;
      },
      set(v: boolean) {
        console.log("Setting share of curr node:", v);
        (props.file as Node).setShared(v);
        emit("data-updated");
      },
    });

    return {
      fetchingMeta,
      close,
      fileSize,
      sharedToggle
    };
  },
  methods: {
    copyShared() {
      (this.$refs.sharedLink as HTMLInputElement).select();
      document.execCommand("copy");
      console.log("copied shared path");
    },
  },
});
</script>

<style scoped>
.card {
  align-self: center;
  width: 100%;
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

@media only screen and (max-width: 768px) {
  #f-info-wrapper {
    position: fixed;
    overflow: hidden;
    display: flex;
    justify-content: center;
    top: 0;
    height: 100vh;
    width: 100vw;
    background-color: rgba(0, 0, 0, 0.1);
  }

  .flyin-enter-active, .flyin-leave-active {
    transition: transform 0.2s ease-out;
  }
  
  .flyin-enter-from, .flyin-leave-to {
    transform: translateX(100vw);
  }
  .flyin-enter-to, .flyin-leave-from {
    transform: translateX(0vw);
  }
}
</style>
