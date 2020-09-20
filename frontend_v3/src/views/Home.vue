<template>
  <main class="container-md">
    <h1>Welcome, {{$store.state.auth.user.name}}</h1>
    <div class="card col-md-3">
      <div class="card-img-top">
        <div class="progress">
          <div class="progress-bar" role="progressbar" :style="{width: interpolatedStorageUsed / 5000 + '%'}" aria-valuenow="25" aria-valuemin="0" aria-valuemax="100"></div>
        </div>
      </div>
      <div class="card-body">
        <h4 class="card-title">Storage used: {{storageUsed}}</h4>
      </div>
    </div>
  </main>
</template>
<script lang="ts">
import { defineComponent } from 'vue'
import { mbToFormattedString } from '../business/utils'
import { store } from '../store' 

export default defineComponent({
  name: 'Home',
  data() {
    return {
      interpolatedStorageUsed: 0
    }
  },
  mounted() {
    const targetStorageUsed = 11.1
    const smooth = () => {
      const delta = targetStorageUsed - this.interpolatedStorageUsed
      if (delta / targetStorageUsed > 0.01) {
        this.interpolatedStorageUsed += delta * 0.05
        requestAnimationFrame(smooth)
      }
      else {
        // finish
        console.log('storageUsed animation finished')
        this.interpolatedStorageUsed = targetStorageUsed
      }
    }

    requestAnimationFrame(smooth)
  },
  computed: {
    userName() {
      store.auth.user.value?.name
    },
    storageUsed() {
      return mbToFormattedString(this.interpolatedStorageUsed)
    }
  }
})
</script>
<style scoped>
h1 {
  text-align: center;
  margin-top: 1em;
}

.progress {
  margin-top: 1em;
}
</style>
