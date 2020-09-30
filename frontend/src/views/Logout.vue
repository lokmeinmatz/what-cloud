<template>
  <main class="container-md">
    <h1>logging out...</h1>
    <div class="progress">
      <div class="progress-bar" :style="{width: `${progress}%`}" role="progressbar" aria-valuenow="0" aria-valuemin="0" aria-valuemax="100"></div>
    </div>
  </main>
</template>
<script lang="ts">
import { defineComponent } from 'vue'
import { store } from '../store'
import * as fs from '../business/fs'
import { delay } from '../business/utils'

export default defineComponent({
  name: 'Logout',
  data() {
    return {
      progress: 0
    }
  },
  async mounted() {
      console.log('starting logout...')
      //store.rootNode.value = null
      fs.reset()
      await delay(100)
      this.progress = 25
      // TODO need to add this to Files.vue?
      //state.nodeInfoDisplay.emit(null)
      await delay(100)
      this.progress = 50
      await fetch('/api/user/logout', {
                headers: {
                    'Authorization': `Bearer ${store.user.value?.authToken}`
                }
            })
      store.user.value = null
      await delay(100)
      this.progress = 75
      // TODO send logout to server
      console.log('logout completed')
      await delay(100)
      this.$router.push('/login')
  }
})
</script>
<style scoped>
h1 {
  text-align: center;
  margin-top: 1em;
}

.card {
  margin-top: 1rem;
  width: 100%;
}

</style>
