<template>
  <div>
    <div class="btn-group" id="display-style-selector" role="group">
      <button
        type="button"
        class="btn"
        :class="
          displayStyle == 'list'
            ? { 'btn-primary': true }
            : { 'btn-secondary': true }
        "
        @click="displayStyle = 'list'"
      >
        List
      </button>
      <button
        type="button"
        class="btn"
        :class="
          displayStyle == 'previews'
            ? { 'btn-primary': true }
            : { 'btn-secondary': true }
        "
        @click="displayStyle = 'previews'"
      >
        Previews
      </button>
    </div>
    <div :class="{'list-group': true, 'previews': displayStyle == 'previews'}">
      <FSItem
        v-for="file in content"
        class="list-group-item"
        :preview="displayStyle == 'previews'"
        :key="file.name"
        :file="file"
        @nodeinforequested="emitNIreq($event)"
      />
    </div>
  </div>
</template>


<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import FSItem from "./FSItem.vue";
import { Folder, Node } from "../business/fs";

enum DisplayStyle {
  List = "list",
  Previews = "previews",
}

export default defineComponent({
  name: "FileList",
  components: { FSItem },
  props: {
    folder: Folder,
  },
  setup(props, {emit}) {
    const content = computed<Node[]>(() => {
      const c = (props.folder as Folder).children;
      if (c == undefined) return [];
      return c.sort((a, b) => {
        if (a.type == "folder" && b.type != "folder") return -1;
        if (a.type != "folder" && b.type == "folder") return 1;
        return a.name.localeCompare(b.name);
      });
    });

    const displayStyle = ref(DisplayStyle.List);


    const emitNIreq = (node: Node) => {
      console.log('emitNIreq')
      emit('nodeinforequested', node)
    }

    return {
      content,
      displayStyle,
      emitNIreq
    };
  },
});
</script>

<style scoped>
#display-style-selector {
  margin: 0.5em;
}


.previews {
  flex-direction: row;
  flex-wrap: wrap;
  border: none;
}
</style>
