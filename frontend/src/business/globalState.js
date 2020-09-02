import { randString } from './utils'

export class Observable {
    constructor(startValue) {
        this.value = startValue
        this.subscribers = new Map
    }

    emit(val) {
        this.value = val
        this.subscribers.forEach((f) => {
            f(this.value)
        })
    }

    /**
     * 
     * @param {function(any)} f 
     * @returns {string} generated Id
     */
    subscribeAnonymous(f) {
        const id = randString(10)
        this.subscribers.set(id, f)
    }

    /**
     * 
     * @param {string} id 
     * @param {function(any)} f 
     * @returns {null | function(any)} null if this was the first subscription with this id, else the previous function
     */
    subscribeWithId(id, f) {
        const old = this.subscribers.get(id)
        this.subscribers.set(id, f)
        return old | null
    }

    /**
     * Deletes the associated subscription
     * @param {string} id
     */
    unsubscribe(id) {
        this.subscribers.delete(id)
    }
}


export const state = {
    nodeInfoDisplay: new Observable(null)
}

window.globalState = state