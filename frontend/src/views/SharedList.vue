<template>
  <div id="shared">
    <main v-if="$store.getters['auth/isLoggedIn']" class="container-sm">
      <h2>Here you can see all your shares</h2>
      <div class="list-group">
        <FSItem
          v-for="entry in sharedEntries"
          class="list-group-item"
          :key="entry.shared"
          :file="entry"
        />
      </div>
    </main>
    <main v-else class="container-sm">
      <h2>You're not logged in!</h2>
      <p>Did you want to see a shared folder? Make sure to copy the whole URL!</p>
      <p>It should have some random characters after /shared/...</p>
    </main>
    <aside :class="{display: nodeInfo != null}">
      <FileInfo class="display" :file="nodeInfo" v-if="nodeInfo != null" @data-updated="updateSharedList" />
    </aside>
  </div>
</template>
<script>
import FSItem from "../components/FSItem";
import PathDisplay from "../components/PathDisplay";
import FileInfo from "../components/FileInfo";
import { updateShared } from "../business/fs";
import { state } from "../business/globalState";

export default {
  name: "SharedList",
  components: {
    FSItem,
    PathDisplay,
    FileInfo,
  },
  async mounted() {
    console.log(this)
    await this.updateSharedList();
  },
  methods: {
    async updateSharedList() {
      if (!this.$store.getters["auth/isLoggedIn"]) return;

      state.nodeInfoDisplay.subscribeWithId("files", (f) => {
        this.nodeInfo = f;
      });

      const url = "/api/shared";
      let res;
      try {
        res = await fetch(url, {
          headers: {
            Authorization: `Bearer ${this.$store.state.auth.user.auth_token}`,
          },
        });
      } catch (e) {
        console.error(e);
        return null;
      }

      if (res.ok) {
        let sharedEntries = await res.json();

        this.sharedEntries = await updateShared(sharedEntries);
      }
    },
  },
  data() {
    return {
      sharedEntries: [],
      nodeInfo: null,
    };
  },
};
</script>
<style scoped>
#shared {
  position: relative;
  display: grid;
  grid-template-columns: auto min-content;
  grid-template-rows: 100%;
  min-height: 100%;
  width: 100vw;
  align-content: stretch;
  overflow-x: hidden;
}

.header {
  margin: 1em 0;
  height: 3em;
  display: grid;
  justify-items: start;
  align-items: center;
  max-width: 100%;
  overflow-x: auto;
}

aside {
  display: flex;
  transition: width ease-in-out 0.2s;
  width: 0;
}

.display {
  width: 20em;
}

@media only screen and (max-width: 768px) {
  #shared {
    grid-template-columns: 1fr;
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