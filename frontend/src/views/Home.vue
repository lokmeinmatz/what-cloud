<template>
  <main class="container-md">
    <h1>Welcome, {{ userName }}</h1>
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
import { computed, defineComponent, onMounted, ref } from 'vue'
import { mbToFormattedString } from '../business/utils'
import { store } from '../store' 

export default defineComponent({
  name: 'Home',
  setup() {
    const interpolatedStorageUsed = ref(0)
    onMounted(() => {
      const targetStorageUsed = 11.1
      const smooth = () => {
        const delta = targetStorageUsed - interpolatedStorageUsed.value
        if (delta / targetStorageUsed > 0.01) {
          interpolatedStorageUsed.value += delta * 0.05
          requestAnimationFrame(smooth)
        }
        else {
          // finish
          console.log('storageUsed animation finished')
          interpolatedStorageUsed.value = targetStorageUsed
        }
      }

      requestAnimationFrame(smooth)
    })
    const storageUsed = computed(() => {
      return mbToFormattedString(interpolatedStorageUsed.value)
    })

    return {
      interpolatedStorageUsed,
      userName: store.user.value?.name,
      storageUsed
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
