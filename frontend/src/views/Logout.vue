<template>
  <main class="container-md">
    <h1>logging out...</h1>
    <div class="progress">
      <div class="progress-bar" :style="{width: `${progress}%`}" role="progressbar" aria-valuenow="0" aria-valuemin="0" aria-valuemax="100"></div>
    </div>
  </main>
</template>
<script>
import { state } from '../business/globalState'
import * as fs from '../business/fs'

function timeout(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

export default {
  name: 'Logout',
  data() {
    return {
      progress: 0
    }
  },
  async mounted() {
      console.log('starting logout...')
      window.rootNode = null
      fs.reset()
      await timeout(100)
      this.progress = 25
      state.nodeInfoDisplay.emit(null)
      await timeout(100)
      this.progress = 50
      this.$store.commit('auth/setUser', null)
      await timeout(100)
      this.progress = 75
      // TODO send logout to server
      console.log('logout completed')
      await timeout(100)
      this.$router.push('/login')
  }
}
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
