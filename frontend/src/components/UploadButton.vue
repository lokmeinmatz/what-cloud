<template>
  <button class="btn" @click="openSelector" :disabled="uploadActive">
    <svg
      class="w-6 h-6"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
      ></path>
    </svg>
    <input
      type="file"
      ref="hiddenInput"
      style="display: none"
      multiple
      accept="*"
      @change="handleUpload"
    />
    <div class="toast-container">
      <!-- upload status toast -->
      <div
        class="toast bg-primary"
        ref="uploadToast"
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
      >
        <div class="toast-header" v-if="uploadStatus != null">
          <strong class="mr-auto"
            >File Upload {{ uploadStatus.currentFileNum }}/{{
              uploadStatus.totalFiles
            }}</strong
          >
          <!--<small>11 mins ago</small>-->
        </div>
        <div class="toast-body" v-if="uploadStatus != null">
          Uploading {{ uploadStatus.currentFile }} ({{
            (uploadStatus.percent * 100).toFixed(1)
          }}%)
        </div>
      </div>
    </div>
  </button>
</template>


<script lang="ts">
import { defineComponent, reactive, ref } from "vue";
import { Folder, Node } from "../business/fs";
import { uploadFiles, UploadStatus, waitForFinish } from "../business/upload";

export default defineComponent({
  name: "UploadButton",
  props: {
    folder: { type: Node, required: true },
  },
  setup(props) {
    const hiddenInput = ref<HTMLInputElement | null>(null);
    const uploadStatus: UploadStatus = reactive({
      currentFile: "<no upload running>",
      currentFileNum: 0,
      rej: null,
      res: null,
      totalFiles: 0,
      percent: 0,
    });
    const uploadActive = ref(false);
    const uploadToast = ref<HTMLDivElement | null>(null);

    function openSelector() {
      hiddenInput.value?.click();
    }
    async function handleUpload(evt: Event) {
      if (
        evt.target instanceof HTMLInputElement &&
        props.folder instanceof Folder &&
        evt.target.files != null
      ) {
        uploadActive.value = true;
        uploadFiles(uploadStatus, props.folder, evt.target.files);
        console.log(uploadToast.value);
        // show toast
        /* eslint-disable @typescript-eslint/no-explicit-any */
        (window as any).$(uploadToast.value).toast({ autohide: false });
        (window as any).$(uploadToast.value).toast("show");
        console.log("showing upload status");
        await waitForFinish(uploadStatus);

        console.log("hiding upload status");
        (window as any).$(uploadToast.value).toast("hide");
        /* eslint-enable @typescript-eslint/no-explicit-any */
        uploadActive.value = false;
      }
    }

    return {
      hiddenInput,
      openSelector,
      handleUpload,
      uploadStatus,
      uploadActive,
      uploadToast,
    };
  },
});
</script>

<style scoped>
.btn {
  background-color: var(--primary);
  width: 3em;
  height: 3em;
  padding: 0.4em;
  color: white;
}

.btn:hover {
  background-color: rgba(128, 128, 128, 0.3);
}

.toast-container {
  position: fixed;
  bottom: 1em;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: -1000;
}

.toast-header {
  background-color: rgba(255, 255, 255, 0.2);
}
</style>
