import { randString } from './utils'

export class Observable {
    constructor(startValue) {
        this.value = startValue
        this.subscribers = new Map
    }

    currentValue() {
        return this.value
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
    nodeInfoDisplay: new Observable(null),
    // set to either {mode: 'files'} or {mode: 'shared', shareID: <id>}
    fileDisplayState: new Observable({mode: 'files'}),
    baseUrl: location.protocol + '//' +location.host
}

window.globalState = state