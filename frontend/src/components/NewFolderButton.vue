<template>
  <button class="btn" @click.prevent="openNameSelector">
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
        d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"
      ></path>
    </svg>
    <StringInsertModal ref="folderNameModal" :config="{title: 'New Folder', confirmText: 'Create new Folder'}" />
  </button>
</template>


<script lang="ts">
import { defineComponent, PropType, ref, watch } from "vue";
import { Folder } from "../business/fs";
import StringInsertModal from "./StringInsertModal.vue";

export default defineComponent({
  name: "NewFolderButton",
  props: {
    folder: { type: Object as PropType<Folder>, required: true },
  },
  components: {
    StringInsertModal,
  },
  setup(props) {
    const folderNameModal = ref<typeof StringInsertModal | null>(null)

    watch(folderNameModal, n => {
      console.log(n)
    })

    return {
      folderNameModal,
      async openNameSelector() {
        try {
          const fname = await folderNameModal.value?.showModal("NewFolder")
          await props.folder.createChildFolder(fname)
        } catch {
          console.log('User canceled folder creation')
        }
      },
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
</style>
