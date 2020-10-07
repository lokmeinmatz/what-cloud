<template>
  <div id="shared" :class="{'display-node-info': nodeInfo != null}">
    <main v-if="user != null" class="container-sm">
      <h2>Here you can see all your shares</h2>
      <div class="list-group" v-if="sharedEntries.length > 0">
        <FSItem
          v-for="entry in sharedEntries"
          class="list-group-item"
          :key="entry.shared"
          :file="entry"
          @nodeinfo-requested="nodeInfo = $event"
        />
      </div>
      <div v-else class="alert alert-primary" role="alert">
        <h4 class="altert-heading text-dark">You didn't share any data!</h4>
        <p class="text-dark">To share, got to the Info-panel of the desired File / Folder and tick 'Share this folder'.</p>
        </div>
    </main>
    <main v-else class="container-sm">
      <h2>You're not logged in!</h2>
      <p>Did you want to see a shared folder? Make sure to copy the whole URL!</p>
      <p>It should have some random characters after /shared/...</p>
    </main>
    <FileInfo v-model:file="nodeInfo" v-if="nodeInfo != null" @data-updated="updateSharedList" />
  </div>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import FSItem from "../components/FSItem.vue";
import FileInfo from "../components/FileInfo.vue";
import { updateShared, Node } from "../business/fs";
import { store } from "../store";

export default defineComponent({
  name: "SharedList",
  components: {
    FSItem,
    FileInfo,
  },
  async mounted() {
    console.log(this);
    await this.updateSharedList();
  },
  methods: {
    async updateSharedList() {
      if (!store.isLoggedIn.value) return;

      const url = "/api/shared";
      let res;
      try {
        res = await fetch(url, {
          headers: {
            Authorization: `Bearer ${store.user.value?.authToken}`,
          },
        });
      } catch (e) {
        console.error(e);
        return null;
      }

      if (res.ok) {
        this.sharedEntries = await updateShared(await res.json());
      }
    },
  },
  data() {
    return {
      sharedEntries: [] as Node[],
      nodeInfo: null,
      user: store.user,
    };
  },
});
</script>
<style scoped>
#shared {
  position: relative;
  display: grid;
  grid-template-columns: 100vw 25em;
  grid-template-rows: 100%;
  min-height: 100%;
  width: 100vw;
  align-content: stretch;
  overflow-x: hidden;
  overflow-x: hidden;
  transition: grid-template-columns ease-out 0.3s;
}

#shared.display-node-info {
  grid-template-columns: calc(100vw - 25em) 25em;
}


@media only screen and (max-width: 768px) {
  #shared {
    grid-template-columns: 100vw auto;
  }

  #shared.display-node-info {
    grid-template-columns: 100vw auto;
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